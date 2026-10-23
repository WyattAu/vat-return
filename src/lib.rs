//! The VAT return, computed from the documents themselves.
//!
//! # The one rule this crate exists to enforce
//!
//! **Only deductible input tax goes in the purchases box.** A bill's gross
//! includes tax the business may never reclaim — entertainment, non-business
//! use, unmet documentation (Article 168). That tax is expense, not a
//! reclaimable input, and returns that net it into the box are overclaims:
//! the kind of error that is small per line, systematic, and expensive when a
//! authority finds it. `ap-kit`'s `Recoverability` split is therefore the
//! *only* source of the input-tax figure here — the crate cannot be handed a
//! raw tax total by mistake, because it takes bills, not numbers.
//!
//! # The boxes (UK MTD numbering, which other regimes resemble)
//!
//! - **Box 1** — VAT due on sales: the output tax on AR documents.
//! - **Box 4** — VAT reclaimable on purchases: the **deductible** input tax.
//! - **Box 6** — total sales excluding VAT.
//! - **Box 7** — total purchases excluding VAT, including the net of bills
//!   whose tax was not deductible (the cost is still a purchase).
//! - **Box 5** — net: Box 1 minus Box 4. Positive is payable; negative is a
//!   reclaim.
//!
//! # Per currency, per period, from the document
//!
//! A return mixing currencies is a number in no currency — the same lesson as
//! the trial balance — so the return carries one currency and refuses
//! documents in another. The tax point is the supply date where present, else
//! the issue date; a document whose tax point falls outside the period is an
//! error, because the caller pre-filtering wrongly is exactly the bug a loud
//! failure prevents from silently shifting tax between periods.
//!
//! # Deliberately out of scope
//!
//! **UK domestic reverse charge filings.** Under EN 16931 a reverse-charge
//! line carries a zero rate, so its tax contributes nothing to Box 1 here;
//! whether the filer adds notional VAT for domestic reverse charge is a
//! per-jurisdiction filing rule, not a property of the documents. The crate
//! reports what is on the documents, which is the part EN 16931 guarantees.

use ap_kit::Bill;
use invoice_kit::Invoice;

/// Why a return cannot be computed from the documents given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VatReturnError {
    /// A document is denominated in a currency other than the return's.
    ///
    /// Mixing currencies produces a total in no currency, so this is refused
    /// rather than converted: the crate does not pick an FX rate, because a
    /// wrong rate in a tax filing is worse than a missing filing.
    CurrencyMismatch {
        /// The return's currency.
        expected: String,
        /// The document's currency.
        found: String,
    },
    /// A document's tax point falls outside the period.
    OutsidePeriod {
        /// The period, `YYYY-MM`.
        period: String,
        /// The document's tax point, `YYYY-MM-DD`.
        tax_point: String,
    },
    /// The period was not a `YYYY-MM` string.
    MalformedPeriod(String),
}

/// The return for one period in one currency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VatReturn {
    /// The period, `YYYY-MM`.
    pub period: String,
    /// The ISO 4217 code every document was denominated in.
    pub currency: String,
    /// Box 1 — VAT due on sales, minor units.
    pub box1_sales_vat: i64,
    /// Box 4 — VAT reclaimable on purchases, minor units.
    pub box4_purchases_vat: i64,
    /// Box 6 — total sales excluding VAT, minor units.
    pub box6_sales_net: i64,
    /// Box 7 — total purchases excluding VAT, minor units.
    pub box7_purchases_net: i64,
}

impl VatReturn {
    /// Box 5 — net VAT: Box 1 minus Box 4. Positive is payable to the
    /// authority; negative is reclaimable from it.
    pub fn box5_net(&self) -> i64 {
        self.box1_sales_vat - self.box4_purchases_vat
    }

    /// Compute the return for `period` from AR invoices and AP bills.
    ///
    /// The tax point of an invoice is its supply date where present, else its
    /// issue date; of a bill, its issue date. Every document is required to
    /// sit inside the period, so a caller that pre-filters wrongly fails
    /// loudly instead of shifting tax between periods.
    pub fn build(
        period: &str,
        currency: &str,
        invoices: &[Invoice],
        bills: &[Bill],
    ) -> Result<Self, VatReturnError> {
        if period.as_bytes().get(4) != Some(&b'-') {
            return Err(VatReturnError::MalformedPeriod(period.to_string()));
        }

        let mut ret = Self {
            period: period.to_string(),
            currency: currency.to_string(),
            box1_sales_vat: 0,
            box4_purchases_vat: 0,
            box6_sales_net: 0,
            box7_purchases_net: 0,
        };

        for invoice in invoices {
            let point = invoice
                .supply_date
                .clone()
                .unwrap_or_else(|| invoice.issue_date.clone());
            require_in_period(period, &point)?;
            require_currency(currency, invoice.currency.alpha)?;
            ret.box6_sales_net += invoice.net_total().units;
            ret.box1_sales_vat += invoice.total_tax().units;
        }

        for bill in bills {
            let point = bill
                .supply_date
                .clone()
                .unwrap_or_else(|| bill.issue_date.clone());
            require_in_period(period, &point)?;
            require_currency(currency, bill.currency.alpha)?;
            let split = bill.split();
            ret.box7_purchases_net += split.net;
            // THE rule: only the deductible portion is reclaimable. The
            // non-deductible part is expense; its net still counts in Box 7.
            ret.box4_purchases_vat += split.deductible_tax;
        }

        Ok(ret)
    }
}

/// `YYYY-MM-DD` inside `YYYY-MM`?
fn require_in_period(period: &str, tax_point: &str) -> Result<(), VatReturnError> {
    if tax_point.get(..7) == Some(period) {
        Ok(())
    } else {
        Err(VatReturnError::OutsidePeriod {
            period: period.to_string(),
            tax_point: tax_point.to_string(),
        })
    }
}

fn require_currency(expected: &str, found: &str) -> Result<(), VatReturnError> {
    if expected == found {
        Ok(())
    } else {
        Err(VatReturnError::CurrencyMismatch {
            expected: expected.to_string(),
            found: found.to_string(),
        })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use ap_kit::{BillLine, Recoverability};
    use double_entry::money::currencies;
    use invoice_kit::{Decimal, InvoiceLine};
    use vat_rules::{DocumentType, RoundingPolicy, TaxCategory};

    fn invoice(net_pence: i64, supplied_on: &str) -> Invoice {
        Invoice::draft("2026-07-01", currencies::GBP, RoundingPolicy::En16931Group)
            .supplied_on(supplied_on)
            .with_line(InvoiceLine {
                id: "1".into(),
                description: "consulting".into(),
                quantity: Decimal::whole(1),
                unit_price: Decimal::new(i128::from(net_pence), 2),
                base_quantity: Decimal::whole(1),
                tax_category: TaxCategory::Standard,
                tax_rate_permille: 200,
            })
    }

    fn bill(net_pence: i64, recoverability: Recoverability) -> Bill {
        Bill::draft("2026-07-05", currencies::GBP, RoundingPolicy::En16931Group).with_line(
            BillLine {
                id: "1".into(),
                description: "supplies".into(),
                net_amount: net_pence,
                quantity: "1".into(),
                unit_price: "0.00".into(),
                tax_category: TaxCategory::Standard,
                tax_rate_permille: 200,
                recoverability,
            },
        )
    }

    #[test]
    fn the_boxes_sum_the_documents_and_only_deductible_tax_reclaims() {
        // Sales: £1,200 + £880 net at 20%.
        let sales = vec![
            invoice(120_000, "2026-07-02"),
            invoice(88_000, "2026-07-08"),
        ];
        // Purchases: £400 fully deductible, £300 half deductible, £200
        // entertainment (not deductible at all).
        let purchases = vec![
            bill(40_000, Recoverability::Full),
            bill(
                30_000,
                Recoverability::Proportion {
                    numerator: 1,
                    denominator: 2,
                },
            ),
            bill(
                20_000,
                Recoverability::None(ap_kit::NonDeductibleReason::Entertainment),
            ),
        ];

        let ret = VatReturn::build("2026-07", "GBP", &sales, &purchases).expect("computes");
        assert_eq!(ret.box6_sales_net, 208_000);
        assert_eq!(ret.box1_sales_vat, 41_600);
        // Box 7 counts every purchase's net, deductible or not.
        assert_eq!(ret.box7_purchases_net, 90_000);
        // Box 4 is deductible only: 80 + 30 + 0 — the entertainment tax is
        // expense, not a reclaim.
        assert_eq!(ret.box4_purchases_vat, 11_000);
        assert_eq!(ret.box5_net(), 30_600, "payable");
    }

    #[test]
    fn a_return_that_reclaims_nothing_is_a_reclaim() {
        let sales = vec![invoice(10_000, "2026-07-02")];
        let purchases = vec![bill(90_000, Recoverability::Full)];
        let ret = VatReturn::build("2026-07", "GBP", &sales, &purchases).expect("computes");
        assert!(ret.box5_net() < 0, "more input than output is a reclaim");
        assert_eq!(ret.box5_net(), -16_000);
    }

    #[test]
    fn a_document_outside_the_period_is_a_loud_error() {
        let sales = vec![invoice(10_000, "2026-08-02")];
        assert_eq!(
            VatReturn::build("2026-07", "GBP", &sales, &[]),
            Err(VatReturnError::OutsidePeriod {
                period: "2026-07".into(),
                tax_point: "2026-08-02".into()
            })
        );
    }

    #[test]
    fn currencies_never_mix_into_a_total_in_no_currency() {
        let mut eur = invoice(10_000, "2026-07-02");
        // Rebuild the same invoice in EUR by writing the currency through the
        // public field.
        eur.currency = currencies::EUR;
        assert_eq!(
            VatReturn::build("2026-07", "GBP", &[eur], &[]),
            Err(VatReturnError::CurrencyMismatch {
                expected: "GBP".into(),
                found: "EUR".into()
            })
        );
    }

    #[test]
    fn reverse_charge_lines_contribute_net_but_no_box_one_vat() {
        // EN 16931: a reverse-charge line carries a zero rate, so the document
        // itself reports no output tax. What the filer adds for UK domestic
        // reverse charge is a filing rule, not a document property — the crate
        // reports the documents.
        let rc = Invoice::draft("2026-07-03", currencies::GBP, RoundingPolicy::En16931Group)
            .with_line(InvoiceLine {
                id: "1".into(),
                description: "EU services".into(),
                quantity: Decimal::whole(1),
                unit_price: Decimal::new(50_000, 2),
                base_quantity: Decimal::whole(1),
                tax_category: TaxCategory::ReverseCharge,
                tax_rate_permille: 0,
            });
        let _ = DocumentType::Invoice;
        let ret = VatReturn::build("2026-07", "GBP", &[rc], &[]).expect("computes");
        assert_eq!(ret.box6_sales_net, 50_000, "the net counts");
        assert_eq!(ret.box1_sales_vat, 0, "the document carries no output tax");
    }

    #[test]
    fn a_malformed_period_is_refused_before_any_document_is_examined() {
        assert_eq!(
            VatReturn::build("July 2026", "GBP", &[], &[]),
            Err(VatReturnError::MalformedPeriod("July 2026".into()))
        );
    }
}
