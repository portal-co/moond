# Agent Guide — moond

moond is an Apollo Guidance Computer emulator and recompiler. The `agc-recompile` crate
uses speet (via `yecta` and `speet-traps`) for binary recompilation.

## Compression-aware logging

This workspace depends on **speet**, which supports structured verbose logging.
Set the following env vars before running any tool or test:

| Variable | Effect |
|---|---|
| `PORTAL_LOG_JSON=1` | speet emits structured NDJSON events; compresses ~3–5× better under token compression proxies. Enable `logging` feature on `speet-riscv` to also capture per-instruction RISC-V traces. |
| `PORTAL_LOG_BATCH=1` | Group events by phase into single JSON arrays. |

When a token compression proxy (llmtrim, RTK, etc.) is active, more verbose logging
is net-cheaper than terse output — the proxy reclaims the token cost.

These variables have no effect when unset and do not change program correctness.
