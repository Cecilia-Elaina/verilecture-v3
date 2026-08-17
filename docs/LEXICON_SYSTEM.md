# Lexicon and textbook system

Supported local source types are text PDFs, DOCX, PPTX, TXT and Markdown.
Scanned PDFs fail closed with a text-layer message; OCR is out of scope for
V3. The local parser extracts metadata candidates, headings, chapters and
term candidates before any optional cloud operation.

The textbook hard limit is `min(10% of extracted Unicode characters, 120000)`.
Only selected short excerpts may be sent after independent consent, and each
excerpt is recorded in a local payload audit. Later exam analysis sends only a
structured lexicon, not textbook prose.

A lexicon is versioned. An audio job stores at most one lexicon ID and the
version snapshot used. Local calibration creates a new transcript version and
does not rewrite raw ASR; numbers, units, formulae and negation are protected.

