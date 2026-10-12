# Provenance — fx fixtures

Source: `docs/harness-configs/fx.md` (see last-verified date in that doc)
Generated: 2026-10-12
Sanitized: no credentials; gateway auth is `AI_GATEWAY_API_KEY` env reference only (fake)
Platform-normalized: LF line endings only.

| Fixture | Kind | Description |
|---|---|---|
| settings.minimal.json | StrictJson | minimal valid settings.json |
| settings.populated.json | StrictJson | populated: provider, permission_mode, workspaces map |
| settings.foreign.json | StrictJson | foreign keys preserved |
| settings.malformed.json | StrictJson | malformed |

All fixtures load via superai_config parsers without panic; malformed variants fail to parse by design.
