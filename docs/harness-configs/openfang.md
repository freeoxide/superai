# OpenFang — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/RightNow-AI/openfang](https://github.com/RightNow-AI/openfang) (Rust "Agent Operating System", v0.6.9, pre-1.0 by its own warning), the README, and the in-repo docs (configuration.md, cli-reference.md, getting-started.md)[1][2][3]. OpenFang runs autonomous scheduled agents ("Hands") on schedules around the clock, connects them to 40 messaging channels, and exposes a dashboard plus an OpenAI-compatible API. Last push 2026-07-02 with the latest release cut 2026-05-12: alive, slower cadence than most of this catalog.

---

## 1. Install, executable, version

```bash
curl -fsSL https://openfang.sh/install | sh      # Windows: install.ps1
openfang init
openfang start                                   # daemon; dashboard at http://localhost:4200
```

## 2. Config files

One TOML file: `~/.openfang/config.toml`. Resolution order: `OPENFANG_HOME` env var, then `~/.openfang` default. The official Docker image sets `OPENFANG_HOME=/data`, so inside that container the config must live at `/data/config.toml`; anything else is silently ignored[2]. `openfang start --config <PATH>` overrides for one run[3].

Secrets never live in config.toml: fields like `api_key_env` and `bot_token_env` hold the name of an environment variable that holds the secret[2]. With no config file at all, OpenFang boots with Anthropic as the default provider and `ANTHROPIC_API_KEY` from the environment[2].

Root keys include `home_dir` (default `~/.openfang`), `data_dir` (default `~/.openfang/data`, SQLite), `api_listen` (default `127.0.0.1:50051`, runtime override `OPENFANG_LISTEN`), and `api_key` (bearer token for the API; empty means unauthenticated local mode)[2].

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `OPENFANG_HOME` | Home directory (config/agents/skills root) | [2] |
| `OPENFANG_LISTEN` | Runtime api_listen override | [2] |
| `OPENFANG_API_KEY`, `OPENFANG_ALLOW_NO_AUTH` | API auth controls | [2] |
| `OPENFANG_AGENTS_DIR` | Agents directory override | [3] |
| `ANTHROPIC_API_KEY`, `GROQ_API_KEY`, `BRAVE_API_KEY`, `TAVILY_API_KEY`, `PERPLEXITY_API_KEY` | Provider/search keys referenced by `api_key_env` fields | [2] |
| `WHATSAPP_WEB_GATEWAY_URL`, `WHATSAPP_GATEWAY_PORT`, `OPENFANG_URL`, `OPENFANG_DEFAULT_AGENT` | WhatsApp web-gateway integration | [1] |

## 4. MCP and A2A

MCP servers are `[[mcp_servers]]` array entries in config.toml with `name`, `timeout_secs` (default 30), `env` passthrough, and a `[mcp_servers.transport]` table selecting `type = "stdio"` (command + args) or `type = "sse"` (url)[2]. An `[a2a]` section toggles the agent-to-agent protocol (`enabled = false` default, `listen_path = "/a2a"`)[2].

## 5. Hands, skills, channels

`openfang hand activate|pause|status|list` manage autonomous Hands; a Hand is a directory with a `HAND.toml` manifest (tools, settings, requirements, dashboard metrics) publishable to FangHub[1]. `openfang skill list|install <SOURCE>` manage the 60 bundled skills plus marketplace; skill development uses SKILL.md format[1][3]. Channel adapters (Telegram, Discord, Slack, WhatsApp, and 36 more) are `[channels.<name>]` config sections with per-channel model overrides, DM/group policies, and rate limiting[1]. `openfang migrate --from openclaw --path ~/.openclaw` imports from OpenClaw[1]. `openfang status [--json]` reports daemon state; `openfang dashboard` opens the UI[3].

## 6. Not verified upstream

- The full config.toml field reference (docs/configuration.md is a complete reference; this file extracts the structural keys, MCP, and env layer).
- ACP support: not mentioned; MCP server mode exists in the CLI crate list[1].
- Headless scripting surface beyond `openfang status --json` (the API is the documented programmatic surface, OpenAI-compatible).
- Release cadence after 2026-05-12 (repo pushed 2026-07-02; no newer tag at compile time).

## Sources

- [1] https://raw.githubusercontent.com/RightNow-AI/openfang/main/README.md: product scope, Hands/HAND.toml, channels, WhatsApp gateway env vars, crates, migrate
- [2] https://raw.githubusercontent.com/RightNow-AI/openfang/main/docs/configuration.md: config.toml path and OPENFANG_HOME resolution, secrets-by-env-name, root keys, mcp_servers schema, a2a
- [3] https://raw.githubusercontent.com/RightNow-AI/openfang/main/docs/cli-reference.md: init/start/status/dashboard/skill commands, --config flag, OPENFANG_AGENTS_DIR
- [4] https://github.com/RightNow-AI/openfang/releases/tag/v0.6.9: latest release (2026-05-12)
- [5] https://github.com/Qumulo/llm-agents/tree/main/packages/openfang: nix index entry (owner RightNow-AI, v0.6.9)
