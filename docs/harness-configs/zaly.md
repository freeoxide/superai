# zaly — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/folke/zaly](https://github.com/folke/zaly) (MIT, Folke Lemaitre; Bun/Node monorepo, alpha), npm `@zaly/cli` (latest 0.0.6, modified 2026-09-08), and the path-resolution source in packages/shared[1][2][3]. zaly is a hackable terminal coding agent: polished TUI, persistent per-workspace sessions, multi-provider login with mid-session model switching, explicit permission presets, and a resource system (commands, skills, packs, themes, TypeScript plugins) at user and project scope. Alpha warning: APIs, configuration, and session formats may change before 1.0[1]. Repo pushed 2026-10-12: alive.

---

## 1. Install, executable, version

```sh
npm install -g @zaly/cli     # Node 22.11+; or: bun add -g @zaly/cli
cd your-project
zaly --yolo
```

## 2. Config files and paths

`/config` in-session edits user or project configuration[1]. The concrete layout comes from the path module (alpha; shape may move)[2][3]:

| Setting | Value |
|---|---|
| User config dir | platform env-path for `zaly` (Linux `~/.config/zaly`), containing `models.json` (ModelProvider overrides) |
| Project dir | `.zaly/` at the project root (config root for project scope) |
| Data | platform data dir (`~/.local/share/zaly` on Linux): `sessions/`, `plugins/` |
| State | `state.json` under the platform state dir (cross-run user state) |
| Whole-root override | `ZALY_ROOT` (single root for config/data/state/log/cache/tmp) |
| Per-key overrides | `ZALY_CONFIG`, `ZALY_DATA`, `ZALY_CACHE`, `ZALY_STATE`, `ZALY_LOG`, `ZALY_TMP` |
| App name | `ZALY_APPNAME` (default `zaly`) |

Project scope also reads `.agents/` locations (the paths module resolves both dotZaly and dotAgents roots)[2].

## 3. Permissions

Presets `strict`, `readonly`, `permissive`, `yolo` (allows every tool call including shell and file writes; the README recommends yolo only with trusted models and projects), refined by per-tool allow/ask/deny rules. One-shot via `--yolo`, persistent via config[1]:

```json
{ "permissions": { "preset": "yolo" } }
```

## 4. Resources, plugins, sessions

- Commands, skills, resource packs, themes, and TypeScript plugins (via `@zaly/plugin`) load at user or project scope; `/resources` toggles them and `/reload` applies changes without a restart[1].
- `/login` authenticates with providers; `/model` browses and switches mid-session; `/effort` tunes reasoning; `/context` inspects what the model sees; `/compact` summarizes older history[1].
- Sessions persist and resume per workspace (`/new`, `/resume`, automatic resume of the most recent)[1].

## 5. Not verified upstream

- The settings file name and key schema inside the config dir (`/config` edits it; the JSON shape was not published).
- MCP support: not mentioned anywhere in the README; nothing to document yet.
- The resource-pack and plugin manifest formats (alpha, "still evolving").
- Version probe (`--version` not documented; npm version is the only pin).

## Sources

- [1] https://raw.githubusercontent.com/folke/zaly/main/README.md: alpha warning, install, permissions presets, actions table, resource system, packages
- [2] https://raw.githubusercontent.com/folke/zaly/main/packages/shared/src/paths/paths.ts: config/models/plugins/sessions/state path layout, project dotZaly/dotAgents
- [3] https://raw.githubusercontent.com/folke/zaly/main/packages/shared/src/paths/env.ts: ZALY_ROOT, ZALY_<KEY> overrides, ZALY_APPNAME, per-platform defaults
- [4] https://registry.npmjs.org/@zaly/cli: npm package 0.0.6 (monorepo packages/cli, modified 2026-09-08)
- [5] https://github.com/Qumulo/llm-agents/tree/main/packages/zaly: nix index entry (npm @zaly/cli pin)
