# Cursor Agent CLI — Configuration Reference

**Compiled:** 2026-10-12 against [cursor.com/docs/cli](https://cursor.com/docs/cli/overview). This file covers the standalone Cursor CLI only: the `agent` binary with historical alias `cursor-agent`, distributed separately from the Cursor IDE since 2025-08. The IDE surface (BYOK quirks, cloud agents, privacy modes) lives in [cursor.md](cursor.md); the two share `~/.cursor/` files but not every behavior.

---

## 1. Install, executable, version

```bash
curl https://cursor.com/install -fsS | bash     # macOS / Linux / WSL
irm 'https://cursor.com/install?win32=true' | iex  # Windows PowerShell
agent --version
```

The installed command is `agent`; `cursor-agent` ships as the historical alias and accepts the same flags[1][2]. No npm or brew install path is documented. Auth: `agent login` (browser; `NO_OPEN_BROWSER=1` prints the URL), `agent status`, `agent whoami`, `agent logout`, or `CURSOR_API_KEY` from Dashboard → API Keys (https://cursor.com/dashboard/api) for CI; `--api-key <key>` is the flag form[3][4].

## 2. Config files

| Path | Purpose | Source |
|---|---|---|
| `~/.cursor/cli-config.json` (Win `$env:USERPROFILE\.cursor\cli-config.json`) | Global CLI settings, pure JSON, `version: 1`, self-repairs missing fields, corrupt file backed up as `.bad` | [5] |
| `<project>/.cursor/cli.json` | Project settings; only `permissions` may be set here | [5] |
| `~/.cursor/mcp.json` | User MCP servers (the only user-scope MCP file) | [6] |
| `<project>/.cursor/mcp.json` | Project MCP servers | [6] |
| `<project>/.cursor/rules/*.mdc`, `AGENTS.md` | Rules; the CLI also reads root `CLAUDE.md` | [7] |

`CURSOR_CONFIG_DIR` relocates `cli-config.json` (Linux/BSD also honors `XDG_CONFIG_HOME` → `$XDG_CONFIG_HOME/cursor/cli-config.json`). Scope is `cli-config.json` only: a live probe on 2026-09-18 (see [cursor.md](cursor.md) §1.2) showed `mcp.json` is still read from `$HOME/.cursor/mcp.json` and `$PWD/.cursor/mcp.json` regardless, so MCP isolation needs per-project files or `HOME` relocation[5][6].

cli-config.json keys (current schema): required `version`, `editor.vimMode`, `permissions.allow`, `permissions.deny`; optional `channel`, `model`, `maxMode`, `notifications`, `hints`, `rewind`, `suggestNextPrompt`, `display.*`, `approvalMode` (`allowlist|auto-review|unrestricted`), `sandbox.mode`, `sandbox.networkAccess`, `network.useHttp1ForAgent`, `attribution.*`[5]. Permission rule syntax: `Shell(git)`, `Read(src/**/*.ts)`, `Write(package.json)`, `WebFetch(host)`, `Mcp(server:*)`[5].

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `CURSOR_API_KEY` | API-key auth for CI/SDK | [4] |
| `CURSOR_CONFIG_DIR` | Config dir override (cli-config.json only) | [5] |
| `NO_OPEN_BROWSER=1` | `agent login` prints URL instead of opening a browser | [3] |
| `HTTP_PROXY`/`HTTPS_PROXY` + `NODE_USE_ENV_PROXY=1`, `NODE_EXTRA_CA_CERTS` | Corporate proxy and custom CA | [5] |
| `CURSOR_WORKER_LABELS_FILE` | Labels file for `agent worker` | [8] |

There is no provider endpoint override: CLI auth is exclusively Cursor backend (browser login or API key), and IDE BYOK does not extend to the CLI[3].

## 4. MCP

Transports stdio (`command`, `args`, `env` with `${env:VAR}`, `envFile`), SSE, and streamable HTTP with OAuth (dynamic or static `auth` block; callbacks `https://www.cursor.com/agents/mcp/oauth/callback` and `http://localhost:8787/callback`)[6]. Manage with `agent mcp list|list-tools|login|enable|disable` against the two mcp.json files[6]. superai can write both files; preserve unmodelled keys.

## 5. Headless / CI

```bash
agent -p "prompt"                       # print mode; proposes changes only
agent -p --force "prompt"               # (--yolo) applies edits
agent -p --output-format text|json|stream-json "..."
agent -p --output-format stream-json --stream-partial-output "..."
agent -p --trust --workspace /repo "…"
agent --model gpt-5 -p "…" ; agent --list-models ; agent models
agent --continue | --resume=<chatId> | resume | ls | create-chat
agent acp                               # ACP server over stdio
```

stream-json events mirror Claude Code: `system/init` (carries `.model`), assistant deltas, `tool_call` started/completed, final `result` with `duration_ms`[2][9]. Other flags: `-H "Name: Value"` custom headers, `--approve-mcps`, `--sandbox enabled|disabled` plus `agent sandbox run --network --allow-paths`, `-w/--worktree [name]` with `--worktree-base`, `--plugin-dir <path>` (repeatable), `--skip-worktree-setup`[8]. Prefixing a message with `&` hands it to a Cloud Agent[1].

## 6. Worker mode

`agent worker start` attaches your machine as a private cloud worker: `--auth-token-file`, repeatable `--worker-dir`, `--management-addr`, `--label`/`--labels-file`/`CURSOR_WORKER_LABELS_FILE`, `--idle-release-timeout`, `--pool`, `--data-dir`, `worker debug [--json]`[8].

## 7. Multi-instance wrappers

Per-profile isolation: distinct `CURSOR_CONFIG_DIR` per instance plus its own `CURSOR_API_KEY`; MCP files need per-project `.cursor/mcp.json` (or `HOME` swap) because they ignore the config-dir override. The wrapper script in [cursor.md](cursor.md) §4.3 applies unchanged.

## 8. Not verified upstream

- Current GA/beta label for the CLI (docs carry none; the Aug 2025 forum post called it beta).
- Where login credentials are stored on disk ("securely stored locally" is the docs' only statement)[3].
- Exit codes (examples check `$? -eq 0`; no documented mapping)[9].

## Sources

- [1] https://cursor.com/docs/cli/overview: install commands, platforms, modes, cloud handoff
- [2] https://cursor.com/docs/cli/headless: print mode, output formats, stream-json events
- [3] https://cursor.com/docs/cli/reference/authentication: login/logout, NO_OPEN_BROWSER, CURSOR_API_KEY, dashboard keys
- [4] https://cursor.com/docs/cli/reference/authentication: API-key env/flag forms
- [5] https://cursor.com/docs/cli/reference/configuration: cli-config.json paths/schema, CURSOR_CONFIG_DIR, XDG, proxy vars, permission syntax
- [6] https://cursor.com/docs/context/mcp: mcp.json paths, transports, OAuth callbacks, agent mcp commands
- [7] https://cursor.com/docs/context/rules: rules files, AGENTS.md, CLAUDE.md read by CLI
- [8] https://cursor.com/docs/cli/reference/parameters: full flag list, worker flags
- [9] https://cursor.com/docs/cli/headless: event field details, exit handling examples
