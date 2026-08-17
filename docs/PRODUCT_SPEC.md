# V3 product contract

VeriLecture V3 imports local classroom audio, runs local ASR, keeps raw
evidence immutable, optionally applies a versioned local lexicon, and produces
chapter-organized exam points with a user-provided text LLM. It is not a live
recorder, course/session manager, cloud ASR client, or general chat app.

The three product ASR tiers are fixed: Qwen3-ASR-1.7B + ForcedAligner, Qwen3-
ASR-0.6B + ForcedAligner, and Fun-ASR-Nano-2512 CPU. Ordinary users choose a
processing tier; model/provider routing is an advanced diagnostic concern.

Simplified Chinese is the default interface locale. Every new user-facing
string must have Chinese and English entries.

