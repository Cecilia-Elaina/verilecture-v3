# Data model

The new database is `verilecture_v3.sqlite`; no old Meetily database is
imported. The initial migration contains settings, provider references,
lexicons, audio records, immutable raw/calibrated transcript segments and
exam points. The normalized job/event/lexicon/audit tables from the master
spec are being added through forward-only migrations as each workflow becomes
functional.

Invariants:

- source audio and raw transcript rows are never overwritten;
- calibrated rows have a distinct version/source and preserve timestamps;
- one audio job has zero or one lexicon association;
- API keys and absolute local paths never enter cloud payloads or exports;
- deleting an app-managed copy is explicit and separate from deleting analysis.

