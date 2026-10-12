# Agent Harness Configuration Bible — Master Index

Research catalog of known harness config surfaces — config files, env vars, provider integration, and multi-instance wrapper techniques — compiled 2026-08-25 by parallel research agents against official docs and repos at that date (claude-desktop + chatgpt-desktop added 2026-09-18; muse-code, qoder-cli, zulu-cli, minimax-code, coco, fx, jules, cursor-agent, kilocode-cli added 2026-10-12, plus the missing letta-code row; 13 tier-2 docs added 2026-10-12: devin, replit-agent, copilot-vscode, oh-my-opencode, oh-my-codex, t3code, orca, bb-app, avante-nvim, codecompanion-nvim, codegeex, terminus, qoder-ide; 18 tier-3 docs added 2026-10-12: every-code, claw-code, agentty, command-code, eca, freebuff, gitclaw, hax, jcode, omo-ai, omp, openfang, ori, prime-agent, reasonix, unreal-agent, vix, zaly). Each file cites its sources inline; where docs are thin or a surface could not be verified, the file flags the gap explicitly rather than inferring. Where a capability is absent (e.g., no BYO endpoint), the file states that explicitly. Ledger support states and last-verified dates live in [03-harness-adapters.md](../plans/03-harness-adapters.md) and `crates/superai-core/src/harness_catalog.rs` (91 surfaces: 26 Full, 46 Constrained, 1 SingleInstance, 7 MigrationOnly, 7 ResearchBlocked, 2 Unsupported, 2 ReadOnly; last_verified spans 2026-08-25 to 2026-09-18, the ledger freshness anchor).

## The Documents

### Terminal / CLI harnesses
| Harness | File | Isolation knob for multi-instance wrappers |
|---|---|---|
| Claude Code | [claude-code.md](claude-code.md) | `CLAUDE_CONFIG_DIR` |
| Codex CLI (OpenAI) | [codex-cli.md](codex-cli.md) | `CODEX_HOME` (+ `$NAME.config.toml` profiles ≥0.134) |
| Gemini CLI (Google) | [gemini-cli.md](gemini-cli.md) | ⚠️ **Retired 2026-06-18** for AI Pro/Ultra/free Code Assist; enterprise licences still served. `GEMINI_CLI_HOME`. Successor: [antigravity-cli.md](antigravity-cli.md) |
| Antigravity CLI (`agy`, Google) | [antigravity-cli.md](antigravity-cli.md) | none documented — `$HOME` swap workaround |
| OpenCode (Anomaly) | [opencode.md](opencode.md) | `OPENCODE_CONFIG` (layer) / `XDG_CONFIG_HOME` (full incl. auth.json) |
| Crush (Charmbracelet) | [crush.md](crush.md) | per-project `crush.json`/crushrc + XDG dirs |
| Aider | [aider.md](aider.md) | `--config` / `--env-file` explicit paths |
| Goose (Block→Linux Fdn) | [goose.md](goose.md) | `GOOSE_PATH_ROOT` |
| Amp (Sourcegraph) | [amp.md](amp.md) | `AMP_API_KEY` + `AMP_SETTINGS_FILE` per instance |
| Qwen Code (Alibaba) | [qwen-code.md](qwen-code.md) | settings-path relocation vars (`QWEN_CODE_*_PATH`) |
| Kimi Code CLI (Moonshot) | [kimi-cli.md](kimi-cli.md) | `KIMI_CODE_HOME` |
| DeepSeek Harness (`dsh`) | [deepseek-harness.md](deepseek-harness.md) | `DSH_HOME`. ⚠️ dev preview, breaking changes expected |
| Grok Build (xAI) | [grok-build.md](grok-build.md) | `GROK_HOME` (+ `GROK_CONFIG` JSON overlay!) |
| Mistral Vibe | [mistral-vibe.md](mistral-vibe.md) | `VIBE_HOME` |
| iFlow CLI | [iflow-cli.md](iflow-cli.md) | Gemini-fork lineage — see file |
| Warp Agent Mode | [warp.md](warp.md) | GUI-bound; honest limits documented |
| Amazon Q Developer CLI | [amazon-q-cli.md](amazon-q-cli.md) | ⚠️ **Sunsetting** — new signups blocked 2026-05-15, EOS 2027-04-30. Superseded by [kiro.md](kiro.md) |
| Kiro CLI (AWS) | [kiro.md](kiro.md) | `KIRO_HOME` + AWS credential vars. No BYO endpoint |
| GitHub Copilot CLI | [copilot-cli.md](copilot-cli.md) | `COPILOT_HOME` + `COPILOT_GITHUB_TOKEN` (GA BYOK via `COPILOT_PROVIDER_BASE_URL`) |
| Junie CLI (JetBrains) | [junie-cli.md](junie-cli.md) | `JUNIE_HOME` (BYOK + credits both supported) |
| Forge (ForgeCode) | [forge.md](forge.md) | `FORGE_CONFIG` dir switch |
| Kode CLI | [kode.md](kode.md) | `KODE_CONFIG_DIR` (also reads `CLAUDE_CONFIG_DIR`) |
| Pi (Earendil Works) | [pi.md](pi.md) | `PI_CODING_AGENT_DIR` |
| gptme | [gptme.md](gptme.md) | workspace separation + `GPTME_LOGS_HOME` |
| Nanocoder | [nanocoder.md](nanocoder.md) | `NANOCODER_CONFIG_DIR` / `NANOCODER_PROVIDERS_FILE` |
| Trae Agent (ByteDance) | [trae-agent.md](trae-agent.md) | `--config-file` + `TRAE_CONFIG_FILE` |
| Plandex 2 | [plandex.md](plandex.md) | env-driven provider switching; per-server `PLANDEX_API_HOST` |
| WorkBuddy / CodeBuddy CLI (cbc) | [workbuddy.md](workbuddy.md) | `CODEBUDDY_CONFIG_DIR`; desktop app GUI-only (documented, not mutated) |
| Muse Code (Meta) | [muse-code.md](muse-code.md) | none documented — settings at `~/.config/muse/settings.json`; `META_API_KEY` per instance |
| Qoder CLI (Alibaba) | [qoder-cli.md](qoder-cli.md) | `QODER_CONFIG_DIR` |
| Zulu CLI (Baidu Comate) | [zulu-cli.md](zulu-cli.md) | none — license-key SaaS; state at `~/.comate-engine`, MCP at `~/.comate/mcp.json` |
| MiniMax Code (`mcode`) | [minimax-code.md](minimax-code.md) | `MINIMAX_DATA_DIR` (user data) / `MCODE_INSTALL_DIR` (install dir) |
| Cortex Code / CoCo (Snowflake) | [coco.md](coco.md) | `SNOWFLAKE_HOME` (whole `~/.snowflake` root, shared with Snowflake CLI) |
| fx (Vercel Labs) | [fx.md](fx.md) | none documented — fixed `~/.fx`; `FX_*` per-process overrides |
| Jules CLI (Google) | [jules.md](jules.md) | none — cloud sessions; separate Google accounts only |
| Cursor Agent CLI | [cursor-agent.md](cursor-agent.md) | `CURSOR_CONFIG_DIR` (cli-config.json only; mcp.json stays under `~/.cursor`) — CLI half of [cursor.md](cursor.md) |
| Kilo CLI | [kilocode-cli.md](kilocode-cli.md) | `KILO_CONFIG_CONTENT` / HOME or XDG relocation (no dedicated dir var) — CLI half of [kilo-code.md](kilo-code.md) |
| Letta Code | [letta-code.md](letta-code.md) | `LETTA_LOCAL_BACKEND_DIR` + `LETTA_BASE_URL` per instance |
| Devin CLI (Cognition) | [devin.md](devin.md) | `--config <PATH>` + `DEVIN_MODEL`/`DEVIN_PERMISSION_MODE`; no config-dir env var documented |
| oh-my-codex (`omx`) | [oh-my-codex.md](oh-my-codex.md) | inherits `CODEX_HOME`; per-project `.omx/` state and worktrees; writes `.codex/config.toml` |
| Oh My OpenCode (plugin) | [oh-my-opencode.md](oh-my-opencode.md) | inherits OpenCode levers; upstream renamed to oh-my-openagent (OmO), plugin fork dormant |
| T3 Code (`t3`) | [t3code.md](t3code.md) | `--base-dir` independent server root; agents keep their own config trees |
| Every Code (`code`/`coder`) | [every-code.md](every-code.md) | `CODE_HOME`; reads legacy `~/.codex` (writes only `~/.code`); Codex-fork half of [codex-cli.md](codex-cli.md) |
| Claw Code (`claw`) | [claw-code.md](claw-code.md) | `CLAW_CONFIG_HOME`; build-from-source only, crates.io `claw-code` is a stub |
| agentty | [agentty.md](agentty.md) | `AGENTTY_HOME` + per-dir `AGENTTY_*_DIR` vars; MCP trust model (project needs user-side approval) |
| Command Code (`cmd`/`cmdc`) | [command-code.md](command-code.md) | none documented; fixed `~/.commandcode`, `cmd -p --output-format json` headless |
| Freebuff CLI | [freebuff.md](freebuff.md) | `FREEBUFF_CONFIG_DIR` (absolute only); default root `~/.config/manicode`; BYOK via OS secret service |
| Gitagent (ledger `gitclaw`) | [gitclaw.md](gitclaw.md) | agent-dir isolation via `--dir`; config IS a git repo (`agent.yaml`, `SOUL.md`, `memory/`) |
| hax | [hax.md](hax.md) | XDG dirs (`XDG_CONFIG_HOME`/`XDG_STATE_HOME`); no MCP/plugins by design |
| jcode | [jcode.md](jcode.md) | none documented; `~/.jcode/config.toml`, cross-reads `~/.claude.json` and Codex MCP |
| OmO (`omo`) | [omo-ai.md](omo-ai.md) | `~/.omo/omo.jsonc` layer walk + `OMO_PROFILE`; CLI rename target of [oh-my-opencode.md](oh-my-opencode.md) |
| omp (oh-my-pi) | [omp.md](omp.md) | `~/.omp/agent/{config,models}.yml`; inherits `.claude`/`.codex`/`.cursor` rules+MCP on first run |
| OpenFang | [openfang.md](openfang.md) | `OPENFANG_HOME` (+ `--config <PATH>` per run); agent-OS daemon, secrets by env-name only |
| Ori (OpenRouter) | [ori.md](ori.md) | `ORI_INSTALL_DIR` (install only); wraps other harnesses via `ori claude/codex/hermes` |
| Prime Agent | [prime-agent.md](prime-agent.md) | `PRIME_AGENT_CODING_AGENT_DIR` (root `~/.prime/agent`); daemon-backed sessions |
| Reasonix | [reasonix.md](reasonix.md) | none documented; `~/.reasonix/config.json` + project `.reasonix/`, DeepSeek-only, Go v2 line |
| Unreal Agent (`uat`) | [unreal-agent.md](unreal-agent.md) | `UNREAL_HARNESS_LLM_*` per launch; prefs at `~/.config/unreal-agent/preferences.json`; reads Codex login |
| vix | [vix.md](vix.md) | `~/.vix` / `./.vix` `providers.json`; `vixd` daemon + isolated `vix` instances |
| zaly | [zaly.md](zaly.md) | `ZALY_ROOT` (or per-key `ZALY_*`); alpha, no MCP yet |

### IDE / editor-integrated agents
| Harness | File | Wrapper notes |
|---|---|---|
| Cursor (IDE + CLI) | [cursor.md](cursor.md) | `CURSOR_CONFIG_DIR`; BYOK limited (chat-only), agent needs Cursor backend |
| Cline | [cline.md](cline.md) | VS Code `--user-data-dir` isolation + `CLINE_DATA_DIR` |
| Roo Code (archived 5/2026) | [roo-code.md](roo-code.md) | same isolation pattern; successor = Kilo |
| Kilo Code | [kilo-code.md](kilo-code.md) | kilo.jsonc global+project; CLI shares OpenCode lineage config |
| Continue.dev (now Cursor's) | [continue-dev.md](continue-dev.md) | global vs workspace `.continue/` layers; `cn` headless CLI |
| Zed + external agents via ACP | [zed-acp.md](zed-acp.md) | register wrapper scripts as separate `agent_servers` entries |
| ZCode (Z.ai) | [zcode.md](zcode.md) | No env override — GUI, fixed `~/.zcode/v2/config.json`. Anthropic + OpenAI custom endpoints |
| Windsurf | [windsurf.md](windsurf.md) | no env override — `--user-data-dir` IDE isolation; managed-backend limits documented |
| Auggie (Augment) | [auggie.md](auggie.md) | proprietary context engine; headless `--print` |
| Hermes Agent (Nous) | [hermes-agent.md](hermes-agent.md) | `HERMES_HOME` + `--profile`; local-install-verified `[L]` tags |
| OpenClaw | [openclaw.md](openclaw.md) | `OPENCLAW_HOME` / `OPENCLAW_CONFIG_PATH`. Daemon, so instances mean running services |
| VS Code Copilot Agent Mode | [copilot-vscode.md](copilot-vscode.md) | `code --user-data-dir` + profiles; writable mcp.json at three scopes; IDE half of [copilot-cli.md](copilot-cli.md) |
| avante.nvim | [avante-nvim.md](avante-nvim.md) | Lua-only config (setup/plugin spec); no config file; MCP via mcphub.nvim |
| codecompanion.nvim | [codecompanion-nvim.md](codecompanion-nvim.md) | Lua `setup()` only; native MCP under `mcp` key; built-in ACP client |
| CodeGeeX (Zhipu AI) | [codegeex.md](codegeex.md) | none documented; host-IDE settings, completion/chat plugin, no MCP |
| Qoder IDE and Desktop | [qoder-ide.md](qoder-ide.md) | UI settings + MCP JSON editor, no documented paths; IDE half of [qoder-cli.md](qoder-cli.md) |
| ECA (Editor Code Assistant) | [eca.md](eca.md) | `~/.config/eca/config.json` + `.eca/` project + `ECA_CONFIG` inline; one config for every editor plugin |

### Desktop apps
| Harness | File | Wrapper notes |
|---|---|---|
| Claude Desktop (Anthropic) | [claude-desktop.md](claude-desktop.md) | **no relocation knob (verified-absent)** — writable `mcpServers` at the fixed per-OS default root; aliasing honestly refused; Linux beta |
| ChatGPT Desktop (Chat + Work + Codex) | [chatgpt-desktop.md](chatgpt-desktop.md) | **no app-level relocation**; read-only on `~/.codex` which belongs to [codex-cli.md](codex-cli.md) — alias via `CODEX_HOME` there; Linux preview |
| Orca (Stably AI) | [orca.md](orca.md) | ADE for agent fleets; per-agent binary/env overrides; `~/.orca/` + repo `orca.yaml` |
| bb (get-bb) | [bb-app.md](bb-app.md) | `BB_DATA_DIR` data-root override + `BB_SERVER_PORT`; ACP agents auto-register |

### Self-hosted / cloud platforms & frameworks
| Harness | File | Wrapper notes |
|---|---|---|
| OpenHands | [openhands.md](openhands.md) | V0 TOML vs V1 env model; `OH_PERSISTENCE_DIR`, docker-per-instance |
| SWE-agent (Princeton) | [swe-agent.md](swe-agent.md) | YAML config files + batch instances w/ different settings |
| Factory Droid | [factory-droid.md](factory-droid.md) | `~/.factory/settings.json` + `customModels` OpenAI-compatible blocks |
| Copilot Coding Agent | [copilot-cli.md](copilot-cli.md) | cloud side has NO BYO endpoint (documented) |
| Replit Agent | [replit-agent.md](replit-agent.md) | cloud workspace; skills at `/.agents/skills/`, MCP via replit.com/integrations (HTTPS only); nothing local |
| Terminus (Terminal-Bench) | [terminus.md](terminus.md) | benchmark agent; runs via Harbor (`--ak`/`--ae` per run); no user config surface |
| Orchestrators: Vibe Kanban · Conductor · Sculptor | [orchestrators.md](orchestrators.md) | THE meta-wrapper doc: how each GUI injects env/creds into spawned harnesses |

## Universal patterns (the cheat-sheet)

1. **Config-dir relocation** is the standard isolation mechanism: `CLAUDE_CONFIG_DIR`, `CODEX_HOME`, `GEMINI_CLI_HOME`, `KIMI_CODE_HOME`, `JUNIE_HOME`, `GROK_HOME`, `GOOSE_PATH_ROOT`, `FORGE_CONFIG`, `KODE_CONFIG_DIR`, `NANOCODER_CONFIG_DIR`, `VIBE_HOME`, `COPILOT_HOME`, `HERMES_HOME`, `PI_CODING_AGENT_DIR`, `CURSOR_CONFIG_DIR`, `OPENCLAW_HOME`, `KIRO_HOME`, `DSH_HOME`, `CODEBUDDY_CONFIG_DIR`, `QODER_CONFIG_DIR`, `SNOWFLAKE_HOME`, `MINIMAX_DATA_DIR`. Relocate → run its auth flow once → you have an independent instance.
2. **Env beats config for providers** almost everywhere: every harness that speaks OpenAI-compatible takes `<PROVIDER>_API_KEY` + some base-URL override (`ANTHROPIC_BASE_URL`, `OPENAI_BASE_URL`, `GOOGLE_GEMINI_BASE_URL`, `GROK_CLI_CHAT_PROXY_BASE_URL`, `COPILOT_PROVIDER_BASE_URL`, `OPENROUTER_API_KEY`…). Wrappers just export different values before `exec`.
3. **Anthropic-compatible endpoints are a de-facto standard too** — Claude Code (`ANTHROPIC_BASE_URL`+`ANTHROPIC_AUTH_TOKEN`), Kimi Code (`anthropic` provider with custom `base_url`), Crush/Kilo/Cline ("anthropic" provider type) all repoint at GLM/OpenRouter-style Anthropic-format gateways.
4. **Inline config injection** where supported: Codex `CODEX_CONFIG`, Grok `GROK_CONFIG` (JSON deep-merge), OpenCode `OPENCODE_CONFIG_CONTENT`, Amp `AMP_SETTINGS_FILE`, Nanocoder `NANOCODER_PROVIDERS_FILE`.
4b. **GUI desktop apps have no relocation knob at all** — [Claude Desktop](claude-desktop.md) (writable `mcpServers`, but at a fixed per-OS root — aliasing honestly refused), [ChatGPT Desktop](chatgpt-desktop.md) (read-only on `~/.codex`; alias via codex-cli's `CODEX_HOME`), ZCode (`~/.zcode/v2/config.json`), Windsurf. Config must be written in place; "instances" mean swapping files, not isolating them.
5. **IDE extensions can't be env-isolated** — wrap VS Code itself: `code --user-data-dir <dir>` gives fully separate extension state (Cline/Roo/Kilo pattern).
6. **Orchestrators are prebuilt wrappers**: Vibe Kanban injects per-profile `ANTHROPIC_BASE_URL`/`AUTH_TOKEN`; Conductor overrides executable paths + Bedrock/Vertex; Sculptor passes env through to containerized Claude Code sessions — see orchestrators.md before rolling your own.

Start here if you want "same harness, two providers side by side": pick the harness's file above → §MULTI-INSTANCE WRAPPERS has a ready script.
