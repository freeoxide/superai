# oh-my-codex (`omx`) — Configuration Reference

**Compiled:** 2026-10-12 against the Yeachan-Heo/oh-my-codex README (33.5k stars, MIT, actively pushed), the npm registry (0.21.9), and the project website. OMX is a workflow layer around the OpenAI Codex CLI: it keeps Codex as the execution engine and adds prompt skills (`$plan`, `$ultragoal`, `$team`, `$deep-interview`, `$ralplan`, `$code-review`), agent teams, HUDs, hooks, and durable state under `.omx/`[1][2]. It wraps the `codex` binary: `omx --madmax` maps to Codex `--dangerously-bypass-approvals-and-sandbox`, `--high`/`--xhigh` map to `model_reasoning_effort`, and fresh setups default to gpt-6-astra[1]. macOS and Linux are the primary platforms; Windows is secondary (psmux/WSL2)[1].

---

## 1. Install, executable, version

```bash
npm install -g oh-my-codex        # requires an installed, authenticated codex on PATH
npm install -g @openai/codex      # only if codex is missing; do not stack this onto a
                                  # Homebrew-owned codex binary (EEXIST risk)
omx setup --scope project --merge-agents   # or --scope user
omx doctor
```

Executable is `omx`. No dedicated `omx --version` command is documented; the HUD prints the running version (e.g. `[OMX#0.21.4]`)[1]. Smoke test from the docs: `codex login status` then `omx exec --skip-git-repo-check -C . "Reply with exactly OMX-EXEC-OK"`[1].

## 2. Config files

| Path | Purpose | Source |
|---|---|---|
| `.omx/` | Plans, logs, memory, durable state; `setup-scope.json` (merge policy), `hooks/*.mjs`, `hud-config.json`, `missions/`, `ultragoal`, `state/team/` | [1] |
| `.omx-config.json` | Model and env routing | [1] |
| `.codex/config.toml`, `.codex/hooks.json` | Codex-side config and hooks that OMX reads and can edit | [1] |
| `~/.codex/AGENTS.md` / `./AGENTS.md` | Durable guidance, user and project scope | [1] |
| `${CODEX_HOME:-~/.codex}/plugins/cache/<marketplace>/oh-my-codex/<version>/` | Plugin cache | [1] |

The critical wrapper fact: OMX writes into Codex's own config (`.codex/config.toml`), so backups and preservation rules that apply to the Codex adapter apply here too (see [codex-cli.md](codex-cli.md)).

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `OMX_ROOT` | State root override | [1] |
| `OMX_AUTO_UPDATE` | Update behavior (0/defer) | [1] |
| `OMX_LAUNCH_POLICY` | direct / tmux / detached-tmux / auto | [1] |
| `OMX_REPO_ROOT`, `OMX_WORKTREE_ROOT`, `OMX_GIT_COMMON_DIR`, `OMX_WORKTREE_SCOPE` | Worktree plumbing | [1] |
| `OMX_CODEGRAPH_MODE`, `OMX_CODEGRAPH_PROJECT_PATH` | Code graph behavior | [1] |
| `OMX_SPARKSHELL_BIN` and `OMX_SPARKSHELL_*` (model, fallback model, instructions file, summary timeout) | Sparkshell sub-process | [1] |
| `CODEX_HOME` | Inherited Codex config root (plugin cache path derives from it) | [1] |

## 4. MCP

The Codex plugin ships optional MCP compatibility servers, disabled by default; the `omx_wiki` MCP server is explicit-compatibility only, with the `omx wiki` CLI as the primary interface[1]. No OMX-specific MCP config file is documented; MCP servers configured for Codex itself follow Codex's config surface.

## 5. Multi-instance wrappers

`omx --worktree=feat/task` gives each mission a disposable git worktree with first-class state management (worktree support landed as a headline feature around v0.16.0 and has been reworked since), which is the documented parallelism story[1]. For provider or account separation, OMX defers to Codex: `CODEX_HOME` relocation and the env routing in `.omx-config.json` are the levers. Skills and hooks live in `.omx/` per checkout, so project isolation is the default.

## 6. Not verified upstream

- A dedicated version-probe command (HUD display is the only documented version surface).
- The full `.omx-config.json` schema (a routing reference doc exists in-repo; the fetched README did not print the keys).
- Windows behavior beyond "secondary" (psmux/WSL2 named, no details).

## Sources

- [1] https://github.com/Yeachan-Heo/oh-my-codex: README covering install, config files, env vars, skills, worktrees, MCP posture
- [2] https://registry.npmjs.org/oh-my-codex: package 0.21.9, repository confirmation
- [3] https://oh-my-codex.dev: project website (docs portal)
