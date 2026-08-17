# Text LLM providers

Cloud is text-only. Audio and cloud ASR are never sent. The adapter boundary
supports OpenAI-compatible chat completions, OpenAI Responses, Anthropic
Messages and Gemini generateContent. API keys are written to Windows Credential
Manager through `keyring`; SQLite stores only a secret reference.

Every structured call must parse JSON, validate its schema and validate segment
IDs, chapter IDs and audio ranges. Prompt material is delimited as untrusted
content and cannot issue system instructions. Request logs contain provider,
model, counts, duration, status and stable error codes, never API keys or raw
content.

Cloud provider connection testing is deliberately a separate acceptance step;
local ASR can be developed and tested without a cloud key.

