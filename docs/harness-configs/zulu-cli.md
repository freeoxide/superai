# Zulu CLI (Baidu Comate) — Configuration Reference

**Compiled:** 2026-10-12. Zulu CLI is Baidu Comate's terminal AI coding agent (文心快码), npm `@comate/zulu`, description "AI Coding Agent", current latest 1.7.21 (MIT since 0.0.11), bin `zulu`, Node >= 20.10.0 per package engines (the in-package README says >= 18)[1][2]. The package is published from Baidu's internal monorepo (`baidu/ide/comate-plugin-host` on icode.baidu.com); there is no public source repo[2]. The vendor's own README ships inside the tarball and is the authoritative doc; comate.baidu.com is a JS app with no readable CLI pages[3].

---

## 1. Install, executable, version

```bash
npm install -g @comate/zulu
zulu --help
zulu --ping          # connectivity check against the service endpoint
```

Install auto-detects ripgrep >= 12 and downloads a prebuilt binary when missing; `rg --version` verifies[1]. The README's Docker baseline is `node:current-alpine3.22` plus `ripgrep`[1]. A `zulu --version` flag is not documented; version detection should read npm metadata (`npm ls -g @comate/zulu`), the same fallback the workbuddy doc uses for cbc.

## 2. Surfaces and state

| Path | Purpose | Source |
|---|---|---|
| `~/.comate-engine/log/zulurun-YYYY-MM-DD.log` | Debug logs (only with `DEBUG=true`; never stdout/stderr) | [1] |
| `~/.comate-engine/store/` | Session history, files prefixed `chat_session_` | [1] |
| `~/.comate/mcp.json` | Global MCP server config | [2] |
| `<project>/.comate/mcp.json` / `mcp.local.json` | Project MCP config (constants in the CLI bundle: `MCP_SETTINGS_DIR=".comate"`) | [2] |
| `<project>/.comate/agents/`, `.comate/rules/`, `.comate/commands/` | Subagents (markdown with metadata incl. `mcpServers`), rules (`.comate/rules/**/*.mdr`; `.cursor/rules/**/*.mdc` also read), commands | [2] |
| `<project>/.comate/hooks.json`, `.comate/memory.json`, `.comate/sandbox.json`, `.comateignore` | Hooks, memory, sandbox config, ignore file | [2] |

`zulu inspect` reports what the current workspace loads: rules, mcp, subagents, commands, skills[1].

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `HTTP_HOST` | Service base URL override (default `https://comate.baidu.com`; Baidu-internal users point it at the SaaS proxy) | [1] |
| `NO_PROXY` | Comma-separated proxy exclusion list | [1] |
| `DEBUG` | Boolean; debug logging to `~/.comate-engine/log/` | [1] |

No config-dir override env var exists in the README or the bundle. Auth is a SaaS license string passed per run (`--license`, required); there is no login flow and no API-key env var[1].

## 4. Running tasks (headless only)

There is no interactive TUI. `zulu run` executes one task and exits[1]:

```bash
zulu run --query="summarize this repo" --license=xxxx
zulu run --query="retry direction" --conversation=conv-abc123 --force
zulu list-model --license=xxxx
```

Key flags[1]: `--license/-l` (required), `--query/-q`, `--attach-file`, `--attach-image`, `--activate-skill`, `--activate-command`, `--activate-subagent`, `--activate-rule` (repeatable activation of preloaded extensions), `--model/-m` (id or display name), `--conversation/-c` (multi-turn; history must exist locally), `--mode Agent|Ask|Plan` (default Agent), `--retry N` plus `--retry-query` (auto-retry rounds), `--display task|legacy|record|event-stream|record-stream`, `--force` (preempt queued same-conversation runs), `--cwd`, `--ripgrep`.

Output modes[1]: `task` (markdown with frontmatter carrying `conversationId`, `sessionStorePath`, `status`; meant for subagent consumption), `legacy` (full JSON, `conversationInfo` + `messages` with `elements` trees of REASON/TEXT/TOOL), `record` (flat `messages[]` trajectory with nested `children`), `event-stream` (newline JSON: `conversation-messages`, `conversation-status`, `partial-message-data`, `partial-message-elements`), `record-stream` (closed blocks only).

Same-conversation runs queue strictly in start order; different conversations never block each other[1].

## 5. MCP, skills, subagents

- MCP: server definitions live in the `.comate/mcp.json` files above (global and project); the CLI bundle reads them with `mcpServers` keys and writes back via structured edits[2]. The exact entry schema was not printed anywhere; read an existing entry before generating one. superai can write these files.
- Skills: `zulu inspect` lists skills discovered from a wide set of per-agent directories in the bundle (`.agents/skills`, `.claude/skills`, `.codex/skills`, `.qoder/skills`, `.trae/skills`, `.cursor/skills`, `.continue/skills`, and more); activation at run time is via `--activate-skill`[1][2].
- Subagents: markdown files under `.comate/agents/` whose metadata may pin `tools`, `mcpServers`, and `parentAgents`[2]; activate with `--activate-subagent`.

## 6. Security note

The README states Zulu runs in "Turbo" mode with an empty blacklist: it executes model-produced instructions by default. Treat `zulu run` output and any file it writes as untrusted, and sandbox runs that touch valuable workspaces[1].

## 7. Not verified upstream

- `zulu --version` output (undocumented; use npm metadata).
- The MCP server-entry JSON schema (only file names and the `mcpServers` key are visible in the bundle).
- Whether a `.comate/skills/` directory is read (the bundle's discovery list does not include one; skills come from other agents' directories).
- Any official web doc page for the CLI (comate.baidu.com renders no readable content; juejin articles exist but are third-party).
- Windows support beyond the README's winget ripgrep instructions.

## Sources

- [1] `README.md` inside `@comate/zulu` 1.7.21 tarball (https://registry.npmjs.org/@comate/zulu/-/zulu-1.7.21.tgz) — install, run flags, display modes, env vars, state paths, Turbo-mode warning
- [2] Dist bundle inspection of the same tarball — `.comate`/`.comate-engine` path constants, mcp.json/mcp.local.json, agents/rules/commands/hooks dirs, skills discovery list
- [3] https://comate.baidu.com/ — product site (JS app, no readable CLI docs; empty fetch 2026-10-12)
