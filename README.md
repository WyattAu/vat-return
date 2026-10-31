# vat-return

VAT return boxes computed from the documents themselves: AR invoices via
[`invoice-kit`](https://crates.io/crates/invoice-kit), AP bills via
[`ap-kit`](https://crates.io/crates/ap-kit), the vocabulary from
[`vat-rules`](https://crates.io/crates/vat-rules).

## The one rule this crate exists to enforce

**Only deductible input tax goes in the purchases box.** A bill's gross
includes tax the business may never reclaim; entertainment, non-business use,
unmet documentation (Article 168). That tax is expense, not a reclaimable
input, and returns that net it into the box are overclaims: small per line,
systematic, and expensive when an authority finds them. The crate takes bills,
not numbers, so it cannot be handed a raw tax total by mistake; `ap-kit`'s
`Recoverability` split is the only source of the input-tax figure.

## The boxes (UK MTD numbering)

- **Box 1**; VAT due on sales: output tax on AR documents.
- **Box 4**; VAT reclaimable on purchases: the **deductible** input tax.
- **Box 6**; total sales excluding VAT.
- **Box 7**; total purchases excluding VAT, including the net of bills whose
  tax was not deductible; the cost is still a purchase.
- **Box 5**; net: Box 1 minus Box 4. Positive is payable; negative is a
  reclaim.

## Per currency, per period, from the document

A return mixing currencies is a number in no currency, so the return carries
one currency and refuses documents in another rather than converting; a wrong
FX rate in a tax filing is worse than a missing filing. The tax point is the
supply date where present, else the issue date; a document outside the period
is a loud error, because a caller that pre-filters wrongly is exactly the bug
that would silently shift tax between periods.

## Deliberately out of scope

UK domestic reverse-charge filings. EN 16931 reverse-charge lines carry a zero
rate, so their tax contributes nothing to Box 1 here; whether the filer adds
notional VAT for domestic reverse charge is a per-jurisdiction filing rule,
not a property of the documents. The crate reports what is on the documents,
which is the part EN 16931 guarantees.

## Licence

MIT OR Apache-2.0.
