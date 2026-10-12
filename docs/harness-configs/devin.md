# Devin CLI (Cognition) — Configuration Reference

**Compiled:** 2026-10-12 against docs.devin.ai (CLI quickstart, commands reference, configuration pages) and devin.ai. Devin started as Cognition's cloud software engineer; the current product also ships a local CLI agent, "a local command-line coding agent with deep Devin Cloud integration"[1][2]. The cloud side (sessions, Playbooks, Secrets, Knowledge) stays on Cognition's infrastructure; the CLI works against local files and can hand a session off to a cloud VM (`--cloud`, `devin cloud`, `devin ssh`)[1][3]. The enterprise angle shows in the config precedence: organization settings outrank everything a user or repo can write[4].

---

## 1. Install, executable, version

```bash
curl -fsSL https://cli.devin.ai/install.sh | bash   # macOS / Linux / WSL
brew install --cask devin-cli                        # macOS
irm https://static.devin.ai/cli/setup.ps1 | iex      # Windows PowerShell only
devin version                                        # same as devin --version
```

Windows installers exist for x86_64 and ARM64; the PowerShell script must run in PowerShell, not Git Bash or CMD[1]. Devin Desktop bundles the CLI for "Legacy Windsurf Enterprise" and "Devin Enterprise" plans[1]. Login is browser-based via `devin auth login`, with `--force-manual-token-flow` for pasting a token on SSH-only machines; `devin setup` is the guided wizard[2].

## 2. Config files

JSON with comment support (JSONC) at two levels[4]:

| Path | Purpose |
|---|---|
| `~/.config/devin/config.json` | User config (Windows: `%APPDATA%\devin\config.json`) |
| `.devin/config.json` | Project config, committed |
| `.devin/config.local.json` | Local overrides, auto-gitignored, for secrets |
| `.devin/mcp_config.json` | Project MCP servers (dedicated file since v3000.3, the "Local 3.6" release; older versions kept them in config.json) |
| `.devin/mcp_config.local.json` | Personal MCP servers |
| `~/.config/devin/AGENTS.md` | Global rules across projects |

Documented top-level keys: `agent` (model choice, user-level only), `permissions` with `allow`/`deny` arrays (`Read(**)`, `Exec(git)`, `Exec(npm run)`), `mcpServers`, `hooks`, `read_config_from`, plus user-only `theme_mode`, `unicode_mode`, `show_path`, `sandbox`[4]. `read_config_from` imports rules, skills, and config from Cursor, Windsurf, Claude Code, GitHub Copilot, OpenCode, and Zed[4][5]. Precedence: org/enterprise settings > session grants > project local > project > user; org settings cannot be overridden[4]. An MDM-distributable machine-wide policy file `system.json` exists for enterprises[6].

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `DEVIN_MODEL` | Default model (same as `--model`) | [2] |
| `DEVIN_PERMISSION_MODE` | Permission mode (same as `--permission-mode`) | [2] |
| `DEVIN_SANDBOX` | Sandbox on/off (same as `--sandbox`) | [2] |
| `WINDSURF_API_KEY` | Credential for the ACP server, checked before stored login | [2] |

No config-dir override env var is documented. The user root sits at `~/.config/devin`, so an `XDG_CONFIG_HOME` relocation probably moves it, but that is inference, not a documented guarantee; verify before relying on it.

## 4. Commands

Subcommands: `auth`, `mcp`, `models`, `rules`, `skills`, `plugins`, `migrate`, `list`/`ls`, `cloud` (plus `cloud drs`), `version`, `acp`, `update`, `sandbox`, `setup`, `doctor`, `airgap doctor`, `ssh`, `forward`, `worker`, `uninstall`[2]. Global flags worth wrapping: `--model`, `--permission-mode normal|auto|accept-edits|smart|dangerous|yolo|bypass|autonomous`, `--sandbox`, `--cloud`, `--continue`/`-c`, `--resume <id>`/`-r`, `--print [PROMPT]`/`-p`, `--prompt-file`, `--config <PATH>`, `--export`, `--respect-workspace-trust` (set false in CI, since print mode cannot answer trust prompts)[2].

## 5. MCP

`devin mcp add <name>` with `-t stdio|http`, `-s local|project|user`, `--url`, `--command`, `-e KEY=VALUE`, `-H/--header`, trailing command after `--`; then `mcp list|get|remove|login|logout|enable|disable`[2]. HTTP transport tries Streamable HTTP first and falls back to SSE on 404/405; OAuth login is per-server[2]. File storage is the `mcp_config*.json` files above, so MCP is a writable, path-known surface.

## 6. Skills, rules, plugins

`devin rules`, `devin skills`, and `devin plugins` are first-class subcommands[2]; the extensibility docs put user-level state under `~/.config/devin/` and project state under `.devin/`[7]. Config import via `read_config_from` pulls in rules, skills, and config from six other harnesses: Cursor, Windsurf, Claude Code, GitHub Copilot, OpenCode, and Zed[4][5]. The exact on-disk layout for skills and plugins was not on the fetched pages; read an existing install before writing.

## 7. Multi-instance wrappers

Documented levers: `--config <PATH>` per invocation, `DEVIN_MODEL`/`DEVIN_PERMISSION_MODE` env overrides, and separate auth via `devin auth login --force-manual-token-flow` or `WINDSURF_API_KEY` for the ACP path[2]. There is no documented equivalent of `CLAUDE_CONFIG_DIR`; full state separation needs an unverified `XDG_CONFIG_HOME` swap or distinct machines.

## 8. Not verified upstream

- Skills/plugins on-disk layout under `~/.config/devin/` (subcommands exist; layout not printed on fetched pages).
- Whether `XDG_CONFIG_HOME` relocates the user config root (path suggests yes, no doc found).
- Where `devin auth login` stores credentials.
- Pricing or free-tier boundaries for CLI use (plan references on fetched pages are enterprise-only).

## Sources

- [1] https://docs.devin.ai/cli: platforms, install commands, local agent description, desktop bundling
- [2] https://docs.devin.ai/cli/reference/commands: subcommands, global flags, env vars, MCP commands, auth flows
- [3] https://devin.ai/cli: positioning, cloud hand-off
- [4] https://docs.devin.ai/cli/extensibility/configuration: config paths, keys, precedence, MCP files
- [5] https://docs.devin.ai/cli/reference/configuration/read-config-from: importing Cursor/Windsurf/Claude config
- [6] https://docs.devin.ai/cli/enterprise/system-config: system.json MDM policy file
- [7] https://docs.devin.ai/cli/extensibility: user-level vs project-level state roots
