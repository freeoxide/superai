# OmO (`omo`, npm `omo-ai`) — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/code-yeongyu/oh-my-openagent](https://github.com/code-yeongyu/oh-my-openagent) (OmO, current line 5.x, latest release v5.1.30, pushed 2026-10-12), npm `omo-ai` (latest 5.1.30), and the in-repo configuration reference[1][2][3]. OmO is the standalone `omo` CLI built on senpi, the project's fork of badlogic/pi-mono; one command, multi-model ("mass" graphs of agents), a git-backed markdown memory, and subscription logins. This is the rename target already recorded in [oh-my-opencode.md](oh-my-opencode.md): the old OpenCode plugin lineage became this CLI, and that file and this one split the plugin surface from the CLI surface. The nix index tracks it as `packages/omo-ai`[4].

---

## 1. Install, executable, version

```bash
curl -fsSL https://get.omo.dev/install.sh | bash    # native binary to ~/.local/bin
npm i -g omo-ai                                      # same OmO via package manager
omo
```

Windows: `irm https://get.omo.dev/install.ps1 | iex`. The npm package named `omo` is someone else's; `omo-ai` is the real one[1]. Update via the install line or `omo update`; `omo setup` migrates from the OpenCode edition or LazyCodex (provider keys, custom providers, MCP servers, skills, model picks), and `omo doctor` reports what the old install left behind[1].

## 2. Config files

One unified JSONC file configures every OmO harness (the OpenCode plugin, Senpi/native, and Codex)[2]:

- User layer: `~/.omo/omo.jsonc` (`omo.json` accepted).
- Project layers: `.omo/omo.jsonc` (then `.omo/omo.json`) in every directory from the working directory up to `$HOME`; nearest project file wins over the user layer. `$HOME` itself is skipped by the walk.

Legacy files (`oh-my-openagent.json[c]`, `oh-my-opencode.json[c]`, `~/.omo/config.jsonc`) are read only by the migration engine, never at runtime; migrations record ids in a `_migrations` array and back sources up under `~/.omo/migration-backup-*/`[2].

Resolution within the merged document: shared base keys, then the `[harness]` block (`[opencode]`, `[native]`, `[senpi]` legacy spelling, or `[codex]`), then `profiles.<name>`, then `profiles.<name>.[harness]`[2]. The `[codex]` block means OmO also writes Codex-harness settings, which touches the codex-cli surface.

Security invariant worth pinning: `mcp_env_allowlist` and `browser_automation_engine.playwright_mcp_args` are honored only from the user layer; project layers cannot extend them[2].

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `OMO_PROFILE` | Selects the `profiles.<name>` layer | [2] |
| `OPENCODE_CONFIG_DIR` | Overrides the OpenCode config directory (profile isolation for the plugin harness) | [2] |
| `OMO_DEBUG` | Debug output | [2] |
| `OMO_SEND_ANONYMOUS_TELEMETRY`, `OMO_DISABLE_POSTHOG`, `OMO_CODEX_SEND_ANONYMOUS_TELEMETRY`, `OMO_CODEX_DISABLE_POSTHOG` | Telemetry control | [2] |
| `OMO_CODEX_GIT_BASH_PATH`, `OMO_CODEX_CONFIG_MIGRATION_DISABLED` | Codex-harness specifics | [2] |

## 4. Models, MCP, skills

`/login` covers Claude, ChatGPT, Kimi, and GLM subscriptions[1]. MCP servers are configured inside `omo.jsonc` (the reference's MCPs section; `[opencode]`-era mcp semantics carried forward)[2]. Built-in skills ship browser use plus `playwright`, `git-master`, `frontend`, `review-work`, `remove-ai-slops`, `security-review`, `team-mode` and others; `disabled_skills` is a shared base key honored by every harness, user and project layers unioned[2].

## 5. Sessions and control

`/thinking <level>`, `/efforts <level>`, `/resume`/`/sessions`; from the shell `omo -c` continues the last session and `omo -r` opens the picker; `/help` lists the rest[1]. "ulw"/"ultrawork" prompt keywords and `mass ulw` drive the multi-agent graph modes[1].

## 6. Not verified upstream

- The full `omo.jsonc` key schema (the reference is long; agents, categories, provider chains, and model profiles sections were not extracted in full).
- Headless/non-interactive mode for scripting (`omo -c`/`-r` are resume flags; a `-p` equivalent was not found in the fetched material).
- MCP config sub-schema details beyond the allowlist invariant.
- License: the README badge says SUL-1.0 while the repo also carries the fork lineage; terms were not read.

## Sources

- [1] https://raw.githubusercontent.com/code-yeongyu/oh-my-openagent/master/README.md: install, omo setup/doctor, sessions, mass/ulw, subscription logins
- [2] https://raw.githubusercontent.com/code-yeongyu/oh-my-openagent/dev/docs/reference/configuration.md: file locations and layer walk, resolution order, migration engine, env vars, skills, allowlist invariant
- [3] https://registry.npmjs.org/omo-ai: npm package (latest 5.1.30, repo field points at oh-my-openagent, modified 2026-10-11)
- [4] https://github.com/Qumulo/llm-agents/tree/main/packages/omo-ai: nix index entry
