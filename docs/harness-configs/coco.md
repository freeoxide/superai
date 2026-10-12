# Cortex Code / CoCo (Snowflake) — Configuration Reference

**Compiled:** 2026-10-12 against [docs.snowflake.com/en/user-guide/cortex-code](https://docs.snowflake.com/en/user-guide/cortex-code/cortex-code-cli). Cortex Code, marketed as CoCo, is Snowflake's coding agent ("Snowflake's CLI for building with AI"); the `cortex` executable is the terminal surface, alongside Snowsight, editor, desktop, and ACP variants[1][2][6]. Requires a paid Snowflake account or the dedicated CoCo CLI trial (standard Snowflake trials excluded), the `SNOWFLAKE.CORTEX_USER` or `SNOWFLAKE.CORTEX_AGENT_USER` database role, network access to the account, and the Snowflake CLI (`snow`) installed[2].

---

## 1. Install, executable, version

```bash
curl -LsS https://ai.snowflake.com/static/cc-scripts/install.sh | sh   # macOS / Linux / WSL
irm https://ai.snowflake.com/static/cc-scripts/install.ps1 | iex      # Windows PowerShell
cortex --version        # -V, --version
cortex update           # or cortex update <version>; autoUpdate setting
```

Installs to `~/.local/bin` (POSIX, added to PATH) or `%LOCALAPPDATA%\cortex` (Windows)[2]. Platforms: macOS arm64/x64, Linux x64/arm64, Windows WSL x64 or native x64; needs bash, zsh, or fish[2]. First run opens a setup wizard that picks or creates a connection in `~/.snowflake/connections.toml` (shared with Snowflake CLI; chmod 600)[2][5].

## 2. Config files

All under `~/.snowflake/cortex/` unless `SNOWFLAKE_HOME` moves the root[3][5]:

| File/Dir | Purpose |
|---|---|
| `settings.json` | Main settings (`compactMode`, `autoUpdate`, `theme`) |
| `permissions.json` | `onlyAllow` tool patterns, `defaultMode` ask/allow/deny, `dangerouslyAllowAll` |
| `mcp.json` | MCP servers (global) |
| `hooks.json` | Hooks |
| `conversations/` | Session files |
| `cache/`, `logs/` (coco.log, coco_errors.log), `memory/`, `agents/`, `skills/`, `commands/`, `remote_cache/` | Caches, logs, persistent memory, extensibility dirs |
| `~/.snowflake/config.toml` / `connections.toml` | Snowflake connections (TOML), shared with Snowflake CLI |

Project settings: `.cortex/settings.json` or `.claude/settings.json` in the working directory[3]. Settings precedence (highest first): managed restrictions, profile overrides, project settings, managed defaults, global user settings, built-in defaults[3]. An org-managed `managed-settings.json` can pin policy (see the managed-settings doc page)[3].

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `SNOWFLAKE_HOME` | Overrides the default `~/.snowflake` root (config-dir relocation) | [3] |
| `CORTEX_AGENT_MODEL` | Model override | [3] |
| `CORTEX_ENABLE_MEMORY` | `true`/`1` enables the memory tool | [3] |
| `COCO_DANGEROUS_MODE_REQUIRE_SQL_WRITE_PERMISSION` | Confirmation for SQL writes in bypass mode | [3] |
| `COCO_LOG_LEVEL` / `COCO_DEBUG` / `COCO_LOG_DIR` | Logging (default INFO; log dir defaults to `$SNOWFLAKE_HOME/cortex/logs`) | [3] |
| `COCO_MCP_TOOL_TIMEOUT_MS` | MCP tool timeout (default 60000) | [4] |

## 4. MCP

Global config `~/.snowflake/cortex/mcp.json`, top-level `mcpServers`, entries with `type` stdio (default) / http / sse, `command`/`args`/`cwd` for stdio, `url`/`headers` for remote, `env` (migrated to the OS keychain on first use), `timeout`, and an `oauth` block (client_id, client_name, redirect_port 8585, scope, authorization_server_url)[4]. `${VAR}`, `${VAR:-default}`, and `$VAR` expansion in every field[4].

```bash
cortex mcp add git-server uvx mcp-server-git
cortex mcp add api-server https://api.example.com --type http
cortex mcp list | get <name> | remove <name> | reconnect | start
```

Disabled-server state persists in `mcp-disabled.json`; OAuth tokens live under keychain entry `mcp_oauth_<server-name>`[4]. Tools surface as `mcp__<server>__<tool>` (64-char cap) and are governed by `permissions.json` wildcards (`"ask": ["mcp__*"]`)[4]. Precedence: admin-enforced, connection profile, user mcp.json, plugins, admin URL allowlist[4]. superai can write `mcp.json`.

## 5. Skills, subagents, hooks, plugins

- Skills: project `.cortex/skills/` or `.claude/skills/`, user `~/.snowflake/cortex/skills/` or `~/.claude/skills/`; `SKILL.md` with YAML frontmatter (`name`, `description`, `tools:` runtime tool ids). 50+ bundled skills (`/skill` browses); invoke with `$<skill-name>`; `/skill list|sync|add`; `cortex skill list|add|update|remove|publish`. The curated catalog lives at [Snowflake-Labs/coco-skills](https://github.com/Snowflake-Labs/coco-skills)[6][7].
- Subagents: markdown with frontmatter (`name`, `description`, `tools`, `model`) in `.cortex/agents/` or `.claude/agents/`, or `~/.snowflake/cortex/agents/`[7].
- Hooks: JSON in `.cortex/settings.local.json`/`.claude/settings.local.json` (local), project settings, `~/.claude/settings.json` (user), `~/.snowflake/cortex/hooks.json` (global); events include PreToolUse, PostToolUse, PermissionRequest, SessionStart/End[7].
- Plugins: bundle skills, hooks, subagents, MCP servers under `plugins/<name>/` with CoCo's plugin mechanism; can declare MCP servers in `plugin.json` without overriding higher-priority entries[4][7].

## 6. Headless / batch

`cortex exec "<prompt>"` (or `exec --file <file>|-`): plan mode always disabled, interactive prompts auto-rejected[5]. `-p/--print` is the single-shot form; `--output-format stream-json` for scripting[5]. Related flags: `-c/--connection`, `-w/--workdir`, `-m/--model`, `--mode standard|code`, `--plan`, `--allowed-tools "Bash(git *)"`, `--disallowed-tools`, `--effort minimal|low|medium|high|max`, `--max-turns`, `--private`, `--bypass`/`--dangerously-allow-all-tool-calls`, `--continue`, `-r/--resume [<id>]`, `--cloud [<workspace>]` (run tools in a Snowflake-managed container), `--github <secret>`[5]. Exit codes: test zero versus non-zero only; no stable mapping is documented[5]. `cortex-code-acp` is a separate doc page for ACP clients[5].

## 7. Not verified upstream

- GA versus preview label: neither the CLI page nor settings page states a status.
- Credential storage details beyond connections.toml and the keychain for MCP env/headers.
- The per-OS path of the org-managed `managed-settings.json` (deferred to the managed-settings page, not fetched).

## Sources

- [1] https://github.com/Snowflake-Labs/coco-skills — "Cortex Code ('CoCo') — Snowflake's CLI for building with AI"
- [2] https://docs.snowflake.com/en/user-guide/cortex-code/cortex-code-cli — install, platforms, prerequisites, first-run wizard
- [3] https://docs.snowflake.com/en/user-guide/cortex-code/settings — file map, precedence, settings/permissions keys, env vars
- [4] https://docs.snowflake.com/en/user-guide/cortex-code/cortex-code-mcp — mcp.json schema, cortex mcp commands, keychain, precedence
- [5] https://docs.snowflake.com/en/user-guide/cortex-code/cli-reference — flags, exec, exit codes, connections.toml, update
- [6] https://docs.snowflake.com/en/user-guide/cortex-code/bundled-skills — bundled skills
- [7] https://docs.snowflake.com/en/user-guide/cortex-code/extensibility — skills/subagents/hooks locations and formats
