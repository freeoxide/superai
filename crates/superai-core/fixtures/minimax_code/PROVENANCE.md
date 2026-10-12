# Provenance — minimax_code fixtures

Source: `docs/harness-configs/minimax-code.md` (see last-verified date in that doc)
Generated: 2026-10-12
Sanitized: no credentials; keys are `{env:VAR}` references only (fake)
Platform-normalized: LF line endings only.

| Fixture | Kind | Description |
|---|---|---|
| config.minimal.yaml | Yaml | minimal valid config.yaml |
| config.populated.yaml | Yaml | populated: minimax_api + custom_provider BYOK |
| config.foreign.yaml | Yaml | foreign keys preserved |
| config.malformed.yaml | Yaml | malformed |

All fixtures load via superai_config parsers without panic; malformed variants fail to parse by design.
