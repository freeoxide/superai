# Command Code (`cmd` / `cmdc`) — Configuration Reference

**Compiled:** 2026-10-12 against npm `command-code` (latest 1.79.2, no repository field; distributed from [commandcode.ai](https://commandcode.ai)) and its hosted docs pages[1][2][3][4][5][6]. Command Code is a coding agent "purpose built for open models", powered by the taste-1 model, with continuous preference learning ("taste learning"). Node.js 22+ is required and the CLI refuses to start on Node 20[2]. The binary is `cmd` everywhere except Windows, where `cmd` is the shell, so the alias is `cmdc`; the full `command-code` works on every platform[1]. npm package was modified 2026-10-08: the project is alive and publishing.

---

## 1. Install, executable, version

```bash
npm i -g command-code
cmd            # Windows: cmdc
cmd --version
cmd login      # browser OAuth; key lands in ~/.commandcode/auth.json
```

`/import claude|codex|cursor|pi|opencode|gemini` imports an existing setup from other agents, and `cmd learn-taste` (or `/learn-taste` in-session) builds the taste profile from imported sessions[2].

## 2. Config files

All under `~/.commandcode/`, with project files in the workspace[3]:

| Path | Purpose | Source |
|---|---|---|
| `~/.commandcode/auth.json` | API key / login (quickstart states this path verbatim) | [2] |
| `~/.commandcode/config.json` | Personal preferences; the CLI updates it itself (model picked with /model, theme, compactMode); legacy lowest-precedence layer for six overlapping keys | [3] |
| `~/.commandcode/settings.json` | User-level behavior defaults | [3] |
| `<project>/.commandcode/settings.json` | Team-shared rules (hooks, permissions); commit it | [3] |
| `<project>/.commandcode/settings.local.json` | Personal project overrides, created on demand | [3] |
| `~/.commandcode/providers.json` | BYOK provider entries (see below) | [4] |
| `~/.commandcode/projects/{project}/mcp.json` | Local-scope MCP (default scope) | [3] |
| `~/.commandcode/cron/jobs.json` | Durable scheduled jobs | [3] |

Top-level settings keys documented: planning/implementation models, attribution, permissions, hooks, MCP servers[3].

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `COMMAND_CODE_API_KEY` | Command-provider API key; overrides auth.json | [3] |
| `CMD_ZDR=1` | Zero-Data-Retention mode; pauses ZDR-incompatible feature models | [3] |
| `CMD_LOCAL_ONLY=1` | Local-only mode (`cmd --local-only`): no telemetry, no cloud features; persistable as `{"localOnly": true}` in config.json | [4] |
| `COMMANDCODE_SKIP_UPDATES` | Skip the auto-updater | [3] |
| `COMMANDCODE_SCRATCHPAD` / `COMMANDCODE_SCRATCHPAD_BASE` | Override the per-session scratchpad | [3] |
| `CMD_TOOLS_ALL_ENABLE` / `CMD_TOOLS_<TOOL>_ENABLE` | Enable blocked tools in headless runs (e.g. `CMD_TOOLS_ASK_USER_QUESTION_ENABLE`) | [5] |

No config-dir relocation variable is documented; the root is fixed at `~/.commandcode`.

## 4. MCP

Three scopes with local > project > user precedence on name collisions[3][6]:

```bash
cmd mcp add --transport http stripe --scope project https://mcp.stripe.com   # writes .mcp.json
cmd mcp add notion -- npx -y @notion/mcp-server                              # stdio after --
cmd mcp list
cmd mcp get notion
```

Files: `~/.commandcode/mcp.json` (user), `<project>/.mcp.json` (project, commit it), `~/.commandcode/projects/<slug>/mcp.json` (local, default). Entries use `mcpServers` with `transport` (alias `type`), `url`/`headers` for HTTP, `command`/`args`/`env` for stdio, `enabled`. `${VAR}` and `${VAR:-default}` resolve at runtime in http headers and stdio env; an unset variable without a default stops that server with an error naming the variable[6].

## 5. BYOK

`/connect` in-session (or the config flow) adds providers to `~/.commandcode/providers.json`: any OpenAI-compatible `/chat/completions`, OpenAI `/responses`, or Anthropic-compatible wire, with Ollama/vLLM/LM Studio called out for local servers. `apiKey` accepts `"$ENV_VAR"`, `"{env:VAR}"`, or `"!command"` (command output), never a pasted raw secret; `false` marks a keyless endpoint[4].

## 6. Headless / CI

```bash
cmd -p "explain this file"                    # -p / --print runs headless
cmd -p "review this diff" --model claude-sonnet-4-6 --effort high
cmd -p "..." --output-format json            # machine-readable stream
cmd -p "..." --config theme=dark --config model=...   # repeatable key=value
cmd -p -c                                    # resume most recent headless session
cmd -r                                       # session picker
```

`--yolo` allows file writes and shell commands; permission modes otherwise apply (`--accept-edits` alias included)[5].

## 7. Not verified upstream

- A config-dir or home relocation env var (nothing documented on the settings page).
- The full settings.json key list (page documents categories, not every key).
- Exit codes for print mode (JSON output is documented; the code table is not).
- Whether the `cmd` binary and the npm `command-code` package ever diverge in version.

## Sources

- [1] https://registry.npmjs.org/command-code/-/command-code-1.79.2.tgz (package README): cmd/cmdc alias, Node 22+, docs links
- [2] https://commandcode.ai/docs/quickstart: cmd login, auth.json path, taste learning, /import
- [3] https://commandcode.ai/docs/settings: file tree, settings/config split, env vars, precedence
- [4] https://commandcode.ai/docs/byok: providers.json, apiKey references, local-only mode
- [5] https://commandcode.ai/docs/headless: -p flags, --config key=value, --output-format json, tool-enable env vars
- [6] https://commandcode.ai/docs/mcp: scopes, file paths, schema, ${VAR} expansion
- [7] https://registry.npmjs.org/command-code: version 1.79.2, modified 2026-10-08, no repository field
