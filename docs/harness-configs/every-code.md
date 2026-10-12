# Every Code (`code` / `coder`) — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/just-every/code](https://github.com/just-every/code) (Apache-2.0), npm `@just-every/code` (latest 0.6.196, matching release v0.6.196 of 2026-09-29), and the Qumulo llm-agents nix index entry `packages/code`[1][4]. Every Code (renamed from plain "Code" at v0.5.0) is a community fork of OpenAI's Codex CLI: same lineage, extra ergonomics (browser integration, multi-agent commands, themes, Auto Drive orchestration). The repo README calls the binary situation out: npm installs `code`, and when another tool already provides `code` (VS Code), the same install also provides `coder`[1]. Ledger id note: this file is `every-code` because a `code` id would collide with the VS Code binary that most developers already have on PATH.

---

## 1. Install, executable, version

```bash
npx -y @just-every/code          # run without installing
npm install -g @just-every/code  # installs `code` (and `coder` alongside VS Code's code)
code --version
```

Auth: "Sign in with ChatGPT" (Plus/Pro/Team plans) inside the TUI, or `export OPENAI_API_KEY=...` for usage-based API-key mode[1].

## 2. Config files

Main config: `~/.code/config.toml`; auth at `~/.code/auth.json`[1]. Backwards compatibility with upstream Codex: Every Code reads both `~/.code/` and `~/.codex/` but only ever writes to `~/.code/`[1]. That is a read-sharing overlap with the codex-cli harness (same shape as the cursor/cursor-agent split), and the README warns that a stale `~/.codex/config.toml` can break Codex itself after a round trip, so wrappers should treat `~/.codex` as read-only foreign state[1].

Documented keys in `config.toml`[1]:

| Key | Purpose |
|---|---|
| `model`, `model_provider` | Active model and provider (providers must expose an OpenAI-compatible API, Chat Completions or Responses) |
| `approval_policy` | `untrusted` / `on-failure` / `on-request` / `never` |
| `model_reasoning_effort` | `low` / `medium` / `high` |
| `sandbox_mode` | e.g. `workspace-write` |
| `[tui.theme]` name | Theme selection (see THEME_CONFIG.md in repo) |
| `[profiles.<name>]` | Named presets of the above keys |
| `[mcp_servers.<name>]` | MCP server tables (see below) |

Project guidance: `AGENTS.md` or `CLAUDE.md` at the project root[1].

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `CODE_HOME` | Override the config directory location | [1] |
| `OPENAI_API_KEY` | API-key auth instead of ChatGPT login | [1] |
| `OPENAI_BASE_URL` | Point at OpenAI-compatible endpoints | [1] |
| `OPENAI_WIRE_API` | Force the built-in provider to `chat` or `responses` wiring | [1] |

## 4. MCP

Configured in `~/.code/config.toml` under `[mcp_servers.<name>]` tables, which the README maps to the JSON `mcpServers` object other clients use[1]:

```toml
[mcp_servers.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/project"]
```

## 5. Multi-agent and orchestration surface

Every Code can orchestrate other CLIs it finds on PATH (`claude`, `agy`, `gemini`, `qwen`) for its `/plan` (consensus), `/solve` (race), and `/code` (worktrees) commands, plus `/auto` Auto Drive with `gpt-5.5` / `gpt-5.4-mini` model defaults[1]. This makes it both a harness and a launcher of other catalogued harnesses; a wrapper managing those tools should expect Every Code to spawn them with its own environment.

## 6. Headless / CI

```bash
code --no-approval "run tests and fix any failures"
code --read-only "analyze code quality and generate report"
code --config output_format=json "list all TODO comments"
```

CLI flags: `--model <name>`, `--read-only`, `--no-approval`, `--config <key=val>` (repeatable), `--oss` (local open-source models), `--sandbox <mode>`, `--debug`, `--version`[1]. Exit codes are not documented.

## 7. Not verified upstream

- Whether `CODE_HOME` also moves `auth.json` and sessions, or only `config.toml` (README says "config directory location" without itemizing).
- Default `model_provider` values beyond `openai` and how custom provider blocks are keyed (README shows `model_provider = "openai"` only).
- MCP config-file discovery besides `~/.code/config.toml` (no project-scope MCP file is documented).
- Exit codes; `--config` key list (README shows `output_format=json` as the one example).

## Sources

- [1] https://raw.githubusercontent.com/just-every/code/main/README.md: install and coder alias, config.toml keys, CODE_HOME and env vars, MCP tables, ~/.codex read-compat, CI flags, orchestration commands
- [2] https://registry.npmjs.org/@just-every/code: npm package and latest version 0.6.196 (checked 2026-10-12)
- [3] https://github.com/just-every/code/releases/tag/v0.6.196: latest release, published 2026-09-29
- [4] https://github.com/Qumulo/llm-agents/tree/main/packages/code: nix index entry that pins the upstream (owner just-every, repo code)
