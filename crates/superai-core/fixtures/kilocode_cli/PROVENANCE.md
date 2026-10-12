# Provenance — kilocode_cli fixtures

Source: `docs/harness-configs/kilocode-cli.md` (see last-verified date in that doc)
Generated: 2026-10-12
Sanitized: no credentials; provider keys are `KILO_<FIELD>` env references only (fake)
Platform-normalized: LF line endings only.

| Fixture | Kind | Description |
|---|---|---|
| kilo.minimal.jsonc | JsonC | minimal valid kilo.jsonc |
| kilo.populated.jsonc | JsonC | populated: model, provider options, mcp |
| kilo.foreign.jsonc | JsonC | foreign keys preserved |
| kilo.malformed.jsonc | JsonC | malformed |

All fixtures load via superai_config parsers without panic; malformed variants fail to parse by design.
