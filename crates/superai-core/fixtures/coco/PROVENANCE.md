# Provenance — coco fixtures

Source: `docs/harness-configs/coco.md` (see last-verified date in that doc)
Generated: 2026-10-12
Sanitized: no credentials; connections live in `~/.snowflake/connections.toml`, not modeled here
Platform-normalized: LF line endings only.

| Fixture | Kind | Description |
|---|---|---|
| settings.minimal.json | StrictJson | minimal valid settings.json |
| settings.populated.json | StrictJson | populated: compactMode, autoUpdate, theme |
| settings.foreign.json | StrictJson | foreign keys preserved |
| settings.malformed.json | StrictJson | malformed |

All fixtures load via superai_config parsers without panic; malformed variants fail to parse by design.
