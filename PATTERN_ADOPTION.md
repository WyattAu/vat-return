# PATTERN_ADOPTION

Pattern relationships for vat-return, declared against the WyattAu
pattern canon per references/adoption-format.md. Rows claim what the
code shows; patterns with no relationship here are omitted.

| Pattern | Status | Evidence |
|---|---|---|
| P11 | partial | src/lib.rs:76 (boxes carried in i64 minor units; box5_net is derived on read at src/lib.rs:89 and never stored) |
| X4 | partial | README.md:14 (box semantics follow UK MTD numbering; the deductible-input-tax rule is restated at src/lib.rs:5) |
