# Provenance — cursor_agent fixtures

Source: `docs/harness-configs/cursor-agent.md` (see last-verified date in that doc)
Generated: 2026-10-12
Sanitized: no credentials; CI auth is `CURSOR_API_KEY` env reference only (fake)
Platform-normalized: LF line endings only.

| Fixture | Kind | Description |
|---|---|---|
| cli-config.minimal.json | StrictJson | minimal valid (`version` required) |
| cli-config.populated.json | StrictJson | populated: editor, permissions, approvalMode |
| cli-config.foreign.json | StrictJson | foreign keys preserved |
| cli-config.malformed.json | StrictJson | malformed |

All fixtures load via superai_config parsers without panic; malformed variants fail to parse by design.
