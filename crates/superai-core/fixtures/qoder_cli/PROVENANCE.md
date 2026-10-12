# Provenance — qoder_cli fixtures

Source: `docs/harness-configs/qoder-cli.md` (see last-verified date in that doc)
Generated: 2026-10-12
Sanitized: no credentials; automation auth is `QODER_PERSONAL_ACCESS_TOKEN` env reference only (fake)
Platform-normalized: LF line endings only.

| Fixture | Kind | Description |
|---|---|---|
| settings.minimal.json | StrictJson | minimal valid user settings |
| settings.populated.json | StrictJson | populated: general, mcpServers, enabledPlugins |
| settings.foreign.json | StrictJson | foreign keys preserved |
| settings.malformed.json | StrictJson | malformed |

All fixtures load via superai_config parsers without panic; malformed variants fail to parse by design.
