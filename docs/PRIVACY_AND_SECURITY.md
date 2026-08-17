# Privacy and security

Audio, raw transcripts, textbook files/full text, lexicons and generated
results remain local. Cloud consent is separate for transcript text, structured
lexicon data and limited textbook excerpts. Consent is revocable and provider
changes require reconfirmation.

Model downloads are HTTPS-pinned to immutable revisions and verified before
atomic install. ZIP extraction rejects paths that escape the staging directory.
API keys use Windows Credential Manager and are never stored in SQLite, logs or
exports. User source files are never deleted by import.

