# fx (Vercel Labs) — Configuration Reference

**Compiled:** 2026-10-12 against [fx.sh/docs](https://fx.sh/docs) and [github.com/vercel-labs/fx](https://github.com/vercel-labs/fx). fx is a small open-source coding agent (Apache-2.0, written in Zig): one native binary, no runtime, running the same core on the laptop, in CI, in an agent sandbox, in editors over ACP, and in JavaScript via libfx[1]. The README marks it "Experimental. Use at your own risk."[1] Platforms: macOS and Linux, x86_64 and arm64[1].

---

## 1. Install, executable, version

```bash
curl -fsSL https://fx.sh/setup.sh | bash     # installs to ~/.local/bin
fx -v                                        # --version
fx upgrade                                   # latest release; background update checks by default
```

Build from source needs Zig 0.16.0 (`zig build`)[1].

## 2. Config files

All JSON, 64 KiB size cap, unknown keys ignored, invalid settings reported at startup, restart after hand-editing[2]:

| Path | Purpose | Source |
|---|---|---|
| `~/.fx/settings.json` | Personal settings (models, permissions, workspaces map) | [2] |
| `<workspace>/.fx.json` | Project/shared settings (restricted key set: `max_agent_steps` 0=unlimited, `max_tool_result_bytes` >= 1024 default 65536, `context` bool, `provider_order` up to 8, `provider_strict`) | [2] |
| `~/.fx/mcp.json` | Global MCP servers (where `/mcp add` writes) | [3] |
| `<workspace>/.mcp.json` | Project MCP servers (`mcpServers` key), approved per workspace before connecting | [3] |
| `~/.fx/themes/`, `~/.fx/sessions/`, `~/.fx/recordings/` | Themes, saved conversations, diagnostics recordings | [1][2] |

Precedence (highest wins): CLI flag, environment variable, workspace-scoped entry under `workspaces` in `~/.fx/settings.json`, global entry, `<workspace>/.fx.json`, default[2]. Env/CLI overrides apply to the current process only and are not written back[2]. Personal keys include `provider` ("gateway" default, "codex", "grok", or custom), `providers` (custom connections), `models`, `permission_mode` (ask / auto / full-access, legacy "yolo"; default auto), `effort`, `auto_compact_percent` (10-80, default 80), `review_model`, `fast_mode`/`ultrafast_mode`, `auto_upgrade`, `update_channel` stable/dev, `theme`, `slash_menu_categories`, `notifications.*`, `statusLine.*`, `workspaces` keyed by absolute path[2].

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `AI_GATEWAY_API_KEY` | Vercel AI Gateway key (`fx setup` configures it) | [2] |
| `VERCEL_OIDC_TOKEN`, `TYPESAFE_API_KEY` | Alternative auth (OIDC; review model service) | [2] |
| `FX_PROVIDER`, `FX_MODEL`, `FX_REVIEW_MODEL` | Per-process overrides | [2] |
| `FX_PERMISSION_MODE`, `FX_MAX_AGENT_STEPS`, `FX_PROVIDER_ORDER`, `FX_PROVIDER_STRICT`, `FX_ULTRAFAST` | Behavior overrides (strict boolean parsing; invalid values fail startup) | [2] |
| `FX_THEME`, `FX_SOUND`, `FX_AUTO_UPGRADE=0`, `FX_NO_OPEN_BROWSER=1` | Interface | [2] |
| `FX_TRACE=1`, `FX_DEBUG_RECORD=1`, `FX_RECORD*`, `FX_DISABLE_KEYCHAIN=1` | Diagnostics | [2] |

No config-dir or home override (`FX_HOME` and the like) is documented; the global root is fixed at `~/.fx`.

Auth surfaces: `fx login [vercel|codex|grok]` (Vercel default), `fx setup` (API key), `fx logout`[1][4].

## 4. MCP

`/mcp add` (TUI) and `fx mcp add <name> <command> [args…]` / `fx mcp add --transport http <name> <url>` write to `~/.fx/mcp.json`; project servers in `.mcp.json` need approval (`fx mcp trust approve <name>`, required before `fx ask` uses them)[3]. Other commands: `fx mcp list [--connect]`, `auth`, `logout`, `remove`, `path`, `trust approve|reject|approve-all|reset`; in-session `/mcp`, `/mcp auth <name> --open`, `/mcp reload`[3][4]. Transports: local stdio and HTTP remote; project entries support env vars; profile entry shape uses `"mcp": {...}` with `bearer_token_env` for tokens[3]. Name collisions: profile wins[3]. Tool schemas load on demand to keep context small[1].

## 5. Skills, subagents, context

Skills load from `.fx/skills`, `~/.fx/skills`, and other agents' folders such as `.claude/skills` and `.agents/skills`; subagents take independent work; `AGENTS.md` supplies project instructions[1]. Sessions save under `~/.fx/sessions/` with compaction of older turns kept on disk (`fx -c` resumes)[1]. Commands run in the user's login shell with their aliases and PATH, and receive `AI_AGENT=fx` in the environment[1].

## 6. Headless / embedding

```bash
fx ask "explain the changes in this repository"
fx ask --json "summarize the failing tests" | jq -r .final_output
fx acp          # ACP server over stdio
```

`--json` works with `ask`, `status`, `doctor`, `permissions`, `models`, `workspace`, `session`, `sessions`, `usage`, `credits`, and `upgrade`[4]. Session commands: `fx resume [last|<id>]`, `fx session <last|id>`, `fx sessions --all|--limit|--cursor`, `fx session migrate|recover`; also `fx pr|issue [--create]`, `fx status`, `fx doctor`, `fx usage`, `fx models`, `fx workspace list|add|remove|clear`[4]. Model flags (`--provider`, `--model`, `--effort`, `--fast`, `--ultrafast`, `--provider-order`, `--provider-strict`) go before `resume` when combined with it[4]. Custom model connections (Ollama, OpenRouter, any OpenAI-compatible endpoint) are documented on the custom-model-connections page and stored under `providers`[1][2]. Exit codes are not documented.

## 7. Not verified upstream

- Verified-absent status of a config-dir relocation env var (nothing in the configuration page or env table; absence inferred, not stated by Vercel).
- Exit codes for `fx ask`.
- libfx API details (see sdk/README.md and fx.sh/docs/lib; not fetched in full).

## Sources

- [1] https://raw.githubusercontent.com/vercel-labs/fx/main/README.md — product, install, login, skills/MCP/subagents, sessions, fx ask, libfx
- [2] https://fx.sh/docs/configure-fx/configuration.md — settings files, precedence, key list, env table
- [3] https://fx.sh/docs/capabilities/mcp.md — mcp.json paths, JSON shapes, trust, commands
- [4] https://fx.sh/docs/using-fx/cli.md — CLI reference, --json commands, version flag
