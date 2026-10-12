# MiniMax Code (`mcode`) — Configuration Reference

**Compiled:** 2026-10-12 against [MiniMax-AI/minimax-code](https://github.com/MiniMax-AI/minimax-code) (MIT) and its in-repo docs[1][2][3][4]. MiniMax Code is MiniMax's open-source terminal coding agent; the CLI is `mcode`, distributed as npm `@minimax-ai/code` (npm `latest` 0.4.12 as of 2026-09-19 per the install doc) and via installer scripts[1][3]. Node 22.19+, 24.2+, 25, or 26 for npm/source installs[3].

---

## 1. Install, executable, version

```bash
curl -fsSL https://filecdn.minimax.chat/public/install.sh | bash   # macOS / Linux / WSL
irm https://filecdn.minimax.chat/public/install.ps1 | iex          # Windows PowerShell
npm install -g @minimax-ai/code@latest --registry=https://registry.npmjs.org/ \
  --ignore-scripts=false --include=optional --allow-scripts=@minimax-ai/code,better-sqlite3
mcode --version
mcode --help
```

The installer puts launchers at `~/.minimax-code/bin/mcode` (POSIX) or `%USERPROFILE%\.minimax-code` (Windows); `MCODE_INSTALL_DIR` relocates the install, `MCODE_NO_MODIFY_PATH` skips the PATH edit[1]. Alpine/musl is unsupported by the one-command installer[1]. Native SQLite (better-sqlite3) is required at runtime; keep optional deps and script permissions on[3]. A native SQLite binding plus GitHub release tarballs (`minimax-code-X.Y.Z.tar.gz`) round out the distribution[3].

## 2. Data directory and config files

User data is separate from the install directory[3]:

| Setting | Value |
|---|---|
| Default data dir | `~/.minimax` (published npm CLI and source builds) |
| Profile selected | `~/.minimax-<profile>` |
| Override env | `MINIMAX_DATA_DIR`, falling back to `MAVIS_DATA_DIR` (non-empty wins) |
| Contains | login state, `config.yaml` (provider config), caches, sessions |

`config.yaml` holds the model/provider setup: `minimax_api` for the official API, `custom_provider` entries for BYOK endpoints (see below), custom auth headers for third-party relays[1][2]. Presentation settings also live in the data directory's `config.yaml`[4].

Project guidance: `mcode init .` generates or updates `AGENTS.md`[1].

## 3. Accounts and BYOK

```bash
mcode login                     # mainland China account
mcode login --region global     # Global account
mcode logout
```

Token Plan requires an account with credits; `/status` and `/provider` inspect and switch models in the TUI[1]. BYOK needs no login[1]:

```bash
export MCODE_PROVIDER_API_KEY=...
mcode provider add --name my-provider --base-url https://example.com/v1 \
  --api-format openai-completions --model my-model \
  --api-key-env MCODE_PROVIDER_API_KEY --use
mcode provider list --json
```

`--api-format` is `openai-completions`, `openai-responses`, or `anthropic-messages`; `--use` connection-tests the first model before saving (nothing saved on failure); `--context-limit`/`--output-limit` set limits per repeated `--model`[1]. Custom providers persist under `custom_provider` in the active profile's `config.yaml`[1].

## 4. MCP

Project scope only: `.mcp.json` at the workspace root of the launch directory, standard `mcpServers` shape, `${VAR}` expansion for credentials[2]. Only the main workspace directory is checked[2]. `/mcp` lists servers (status `configured` before connection), `/mcp reload` re-reads after edits; missing env vars surface as `error` naming the variable[2]. No global MCP file is documented; superai can write project `.mcp.json`.

## 5. Skills, plugins, hooks

- Skills: workspace `.agents/skills`, `.claude/skills`, and `.minimax/skills` (directory symlinks supported for whole roots or single skills); `SKILL.md` watched for edits; `/skills [filter]` lists; ACP clients receive enabled skills as slash commands (`/plugin:review` style names)[4].
- Plugins: `/plugins` panel or `mcode plugin` shell commands; official marketplace plus local directories. `mcode plugin marketplace list`, `plugin add <name>@official`, `plugin enable|disable|remove`, `plugin marketplace upgrade`; installing from official requires MiniMax login[2].
- Hooks: synchronous hooks can emit TUI `systemMessage` notices (docs/hooks.md)[2].
- Sessions: `mcode --continue` resumes the latest workspace session, `--session` opens the picker, `/sessions` in TUI[1].

## 6. Headless, ACP, modes

| Entry point | Command | Use |
|---|---|---|
| Interactive TUI | `mcode [prompt]` | conversation, review changes/permissions |
| Headless | `mcode exec [prompt]` | scripts, CI, batch, evals |
| ACP | `mcode acp` | Agent Client Protocol clients |

`--mode lightweight` starts a no-tools conversational session (small system prompt, no tool schemas, no workspace instructions/skills/MCP); standard mode is the default and the mode is fixed at root-session creation; lightweight cannot combine with `--session`/`--continue`[1]. Exit codes are not documented.

## 7. Not verified upstream

- Exact schema of `config.yaml` beyond the `custom_provider`/`minimax_api` distinction and custom auth headers (the file is documented by reference, not by a published key list).
- A global (non-project) MCP config path: docs describe only workspace `.mcp.json`.
- Exit codes for `mcode exec`.
- Telemetry opt-out mechanics (a `mcode telemetry status` command exists on source builds but not on npm 0.4.12; the telemetry doc page was not fetched)[3][4].

## Sources

- [1] https://raw.githubusercontent.com/MiniMax-AI/minimax-code/main/README.md: install, login regions, BYOK, modes, uninstall, env vars
- [2] https://raw.githubusercontent.com/MiniMax-AI/minimax-code/main/docs/examples.md: `.mcp.json` walkthrough, plugin commands
- [3] https://raw.githubusercontent.com/MiniMax-AI/minimax-code/main/docs/installation.md: data dirs, MINIMAX_DATA_DIR/MAVIS_DATA_DIR, GitHub archives, source builds
- [4] https://raw.githubusercontent.com/MiniMax-AI/minimax-code/main/docs/tui-capabilities.md: skills dirs, plugins, MCP/permissions coverage
- [5] https://agent.minimax.io/docs/cli/quick-start: official quick start (linked from README; content lives in repo docs)
