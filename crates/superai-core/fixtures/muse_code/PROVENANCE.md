# Provenance — muse_code fixtures

Source: `docs/harness-configs/muse-code.md` (see last-verified date in that doc)
Generated: 2026-10-12
Sanitized: no credentials; auth is `META_API_KEY` env reference only (fake)
Platform-normalized: LF line endings only.

| Fixture | Kind | Description |
|---|---|---|
| settings.minimal.json | StrictJson | minimal valid (`schema_version` required) |
| settings.populated.json | StrictJson | populated: permissions, tui, mcp_servers |
| settings.foreign.json | StrictJson | foreign keys preserved |
| settings.malformed.json | StrictJson | malformed |

All fixtures load via superai_config parsers without panic; malformed variants fail to parse by design.
