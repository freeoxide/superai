# Provenance — zulu_cli fixtures

Source: `docs/harness-configs/zulu-cli.md` (see last-verified date in that doc)
Generated: 2026-10-12
Sanitized: no credentials; auth is a per-run `--license` flag, never stored here
Platform-normalized: LF line endings only.

| Fixture | Kind | Description |
|---|---|---|
| mcp.minimal.json | StrictJson | minimal valid global mcp.json |
| mcp.populated.json | StrictJson | populated mcpServers entry |
| mcp.foreign.json | StrictJson | foreign keys preserved |
| mcp.malformed.json | StrictJson | malformed |

All fixtures load via superai_config parsers without panic; malformed variants fail to parse by design.
