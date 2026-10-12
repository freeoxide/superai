# Qoder CLI (Alibaba) — Configuration Reference

**Compiled:** 2026-10-12 against [docs.qoder.com/cli](https://docs.qoder.com/cli/quick-start). Qoder CLI is the terminal agent of the Qoder agentic coding platform (from Alibaba; the docs site footer names Bright Zenith Private Limited as the copyright entity)[1]. Platforms: macOS, Linux, Windows (arm64 and amd64; Windows on arm64 unsupported; Node >= 20 only for the legacy npm path)[2]. Separately, a mainland-China edition "Qoder CLI CN" is documented on Alibaba Cloud (help.aliyun.com) under the former Lingma brand; this file covers the global edition unless noted[3].

---

## 1. Install, executable, version

```bash
curl -fsSL https://qoder.com/install | bash          # macOS / Linux (recommended)
irm https://qoder.com/install.ps1 | iex               # Windows PowerShell
npm install -g @qoder-ai/qodercli                     # legacy npm path (not recommended)
qoder --version
```

Upgrade: rerun the script with `--force`, or `qoder update`, or `npm install -g @qoder-ai/qodercli@latest`. Auto-update defaults on; disable with `"general.enableAutoUpdate": false`[2]. First run walks you through sign-in; `qoder login` is the CLI subcommand[1][6].

## 2. Config files

| Level | Path | Source |
|---|---|---|
| User | `~/.qoder/settings.json` | [4] |
| Project | `<project>/.qoder/settings.json` | [4] |
| Local | `<project>/.qoder/settings.local.json` (gitignore it) | [4] |
| Project MCP | `<project>/.mcp.json` (committable) | [5] |

A JSON schema ships at `schemas/settings.schema.json`[4]. Documented key groups[4]: top-level `mcpServers`, `autoMemoryEnabled`, `outputStyle`, `language`, `agent`, `enabledPlugins`, `hooks`; `general` (preferredEditor, vimMode, defaultPermissionMode, enableAutoUpdate, maxAttempts, fileCheckpointing, plan, sessionRetention); `ui`; `model`/`modelConfigs` (name, reasoningEffort, contextWindow, aliases, overrides); `context` (default instruction file `AGENTS.md`, memoryBoundaryMarkers, fileFiltering); `tools`; `mcp` (allowed, excluded, enableAllProjectMcpServers, lazyLoad); `security` (folderTrust, disableYoloMode, environmentVariableRedaction); `permissions` (allow/ask/deny, additionalDirectories, trustDirectories); `privacy`; `advanced`; `statusLine`; `spellcheck`. The full key set is not published; expect unknown keys and preserve them on write. `/init` writes a project `AGENTS.md`[1].

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `QODER_CONFIG_DIR` | Relocates the whole config root (default `~/.qoder`) | [2][4] |
| `QODER_PERSONAL_ACCESS_TOKEN` | Auth token for automation | [4] |
| `QODER_MODEL` | Model override | [4] |
| `QODER_WORKING_DIR`, `QODER_SESSION_ID`, `QODER_SESSION_NAME` | Session/workdir control | [4] |
| `QODER_PERMISSION_MODE`, `QODER_APPEND_SYSTEM_PROMPT` | Headless knobs | [4] |
| `QODER_MCP_LAZY`, `QODER_SUBAGENT_MODEL`, `QODER_MEMORY`, `QODER_MEMORY_USER`, `QODER_ASR_URL` | Feature switches | [4] |
| `HTTP_PROXY`/`HTTPS_PROXY`/`NO_PROXY`, `NODE_EXTRA_CA_CERTS`, `SSL_CERT_FILE` | Network | [4] |

## 4. MCP

Managed by `qoder mcp add <name> [--scope user|local|project] [--transport stdio|sse|http|ws] -- <cmd> <args…>`, `qoder mcp list`, `qoder mcp remove <name>`; `/mcp reload` re-reads in session[5]. Storage per scope: user config in `~/.qoder/settings.json`, local in `<project>/.qoder/settings.local.json`, project in `<project>/.mcp.json`[5]. Tools surface as `mcp__<server>__<tool>` and pass through the permissions system (`"permissions": {"allow": ["mcp__context7__*"]}`)[5]. Stdio is the default transport; with the shell-command prefix enabled, stdio servers launch through a single configured wrapper executable[5]. The full server-entry JSON shape is not printed on the fetched pages; read one written entry before generating.

## 5. Skills, commands, plugins, subagents

- Skills and commands: plugin docs reference `~/.qoder/skills/` and `~/.qoder/commands/` as the global layout that plugin `skills/` and `commands/` directories mirror[7].
- Plugins: a directory with `.qoder-plugin/plugin.json` manifest (only `name` required), bundling `commands/`, `agents/`, `skills/`, `hooks/hooks.json`, `output-styles/`, `bin/`, `.mcp.json`[7]. Managed via `qoder plugins install|uninstall|enable|disable|list|validate|update`, `qoder plugins marketplace add|list|update|remove`; enabled state persists in `settings.json` as `"name@local": true` or `"name@marketplace": true`[7]. Hook scripts receive `QODER_PLUGIN_ROOT` and `QODER_PLUGIN_DATA`[7].
- Subagents and hooks have dedicated docs pages (/cli/subagent, /cli/hooks).

## 6. Headless / CI

`qoder -p "<prompt>"` (`--print`) is the non-interactive entry; `-o text|json|stream-json`, `--input-format stream-json`, `--max-turns`, `--max-output-tokens`, `--no-session-persistence` (print mode only)[6]. Sessions: `--continue`/`-c`, `--resume`/`-r [id]`, `--session-id`, `--fork-session`, `--list-sessions`[6]. Permissions: `--permission-mode default|plan|auto|bypass_permissions|accept_edits|dont_ask`, `--yolo`/`--dangerously-skip-permissions`[6]. Tools: `--tools`, `--allowed-tools`, `--disallowed-tools`, `--mcp-config`, `--strict-mcp-config`, `--allowed-mcp-server-names`[6]. Also `--settings <json|file>`, `--setting-sources`, `--plugin-dir`, `--config-dir`, `--worktree [name]`, `--agent`/`--agents`, `--acp` (ACP server for editors such as Zed)[6]. Docs example: `qoder -p -o json "List project dependencies"`[6]. Exit codes are not documented.

## 7. Multi-instance wrappers

`QODER_CONFIG_DIR` relocates the config tree per instance and `QODER_PERSONAL_ACCESS_TOKEN` or `--settings` isolate credentials; this is the standard relocation pattern in this catalog[2][4]. Project files travel with the checkout; `--setting-sources user` keeps hermetic runs unaffected by repo-local settings[6].

## 8. Not verified upstream

- The exact JSON shape of an `mcpServers` entry in Qoder's own settings files (only the permissions example and scope table are on the fetched pages).
- Whether `settings.json` tolerates comments (JSONC); the schema file exists but no tolerance statement was found.
- Global edition sign-in methods and token storage path (/cli/authentication was not fetched).
- Qoder CLI CN differences beyond its existence and quick-start page on help.aliyun.com[3].
- Exit codes.

## Sources

- [1] https://docs.qoder.com/cli/quick-start: quick start, sign-in flow, slash commands, footer entity
- [2] https://docs.qoder.com/cli/installation: platforms, install scripts, QODER_CONFIG_DIR, uninstall
- [3] https://help.aliyun.com/: Quick Start for Qoder CLI CN (Alibaba Cloud docs)
- [4] https://docs.qoder.com/cli/settings-reference: settings files, key groups, env var table
- [5] https://docs.qoder.com/cli/mcp-servers: MCP scopes, storage paths, transports, CLI commands, permissions example
- [6] https://docs.qoder.com/cli/cli-reference: full flag list, subcommands
- [7] https://docs.qoder.com/cli/plugins: plugin layout, ~/.qoder/skills, ~/.qoder/commands, marketplace
