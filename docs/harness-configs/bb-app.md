# bb (get-bb) — Configuration Reference

**Compiled:** 2026-10-12 against the get-bb/bb repository README and docs/configuration.md, the npm registry (`bb-app` 0.46.0), and the launch write-up at sawyerhood.com. bb is "the agent IDE that builds itself": an open-source agentic IDE where much of the shipped functionality is built with its own extension system. Desktop app, web app, CLI, and HTTP API all drive the same threads, and work can be handed off between agents mid-thread[1][2]. It uses the provider CLIs you already have authenticated (Claude Code, Codex, and any ACP agent) rather than proxying models itself[1][3]. MIT, active, from Sawyer Hood and Michael Yong's passion project[2][3].

---

## 1. Install, executable, version

```bash
npx bb-app@latest          # any platform; web app at http://localhost:38886
```

Desktop builds come from GitHub Releases: macOS Apple Silicon is the stable target, Linux AppImage and Windows x64 are alpha[1]. npm 12+ blocks install scripts, so a first run may need `--allow-scripts=better-sqlite3,node-pty,@parcel/watcher`[1]. A "bb Nightly" desktop build tracks main[1]. No version-probe command is documented; the npm package version (0.46.0) and the update record file below are the version surfaces[4].

## 2. Config files

Default root is `~/.bb/`; `<dataDir>` below is the data directory, which `BB_DATA_DIR` overrides[3]:

| Path | Purpose | Source |
|---|---|---|
| `~/.bb/config.json` | Non-secret settings via `bb-app config set/list/unset`; `customModels` is a manual JSON edit; installer-managed `machineCredential`/`connectMachineId` | [3] |
| `~/.bb/env.json` | Provider credentials and provider env via `bb-app env set/list/unset` | [3] |
| `~/.bb/client.json` | Client-local SSH target mappings for remote editors | [3] |
| `<dataDir>/bb-app-runtime.json` | Runtime state of a running launcher (PID, URL, version); never edit | [3] |
| `<dataDir>/bb-app-update.json` | In-app update outcome record | [3] |
| `<dataDir>/theme/<name>/theme.css` | Custom themes as CSS variable overrides | [3] |
| `<dataDir>/plugins/<id>/` | Per-plugin `data.db`, `secrets/`, `logs/` | [3] |
| `<dataDir>/AGENTS.md`, `<workspace>/.bb/AGENTS.md` | Agent instructions, user and project scope | [3] |
| `<dataDir>/skills/`, `<workspace>/.bb/skills/` | Skills (`SKILL.md` files), user and project scope | [3] |

Precedence: launcher flags, then persistent config/env/client values, then shell environment, then built-in defaults[3]. Dev checkouts get a data directory under `~/.bb-dev/<checkout-instance>/` instead[3].

## 3. Environment variables

Startup-only (full restart required): `BB_APP_SURFACE`, `BB_APP_URL`, `BB_DATA_DIR`, `BB_DEV_APP_PORT`, `BB_EXTERNAL_URL`, `BB_HOST_DAEMON_PORT`, `BB_INHERITED_SKILLS_ROOTS`, `BB_LOG_LEVEL`, `BB_MARKETPLACE_URL`, `BB_POSTHOG_API_KEY`, `BB_TELEMETRY`, `BB_SERVER_BIND_HOST` (only `127.0.0.1` or `0.0.0.0`), `BB_SERVER_PORT` (default 38886), `BB_HOST_DAEMON_PORT` (default 38887), and `BB_FF_*` feature flags[3]. Runtime: `BB_SERVER_URL` (default `http://127.0.0.1:38886`) targets an already-running server from the CLI or daemon[3]. Also `BB_CLI_ERROR_LOG=0`, `FORCE_COLOR`/`NO_COLOR`, and `BB_INSTALL_SKIP_SERVICE=1` for a detached daemon without a service[3]. The inference keys `BB_INFERENCE`, `BB_INFERENCE_FALLBACK`, and `BB_TRANSCRIPTION` were removed and are refused by `bb-app config set`[3]. `BB_TELEMETRY=false` opts out of telemetry[1].

## 4. Agents (ACP) and MCP

- Known agents auto-register when installed: `acp-opencode`, `acp-omp` (oh-my-pi), `acp-grok`, `acp-hermes-agent`; `acp-cursor` is always listed[3].
- A second built-in plugin, "ACP providers (new adapter)", enables `bb acp registry|add|remove`; only one ACP plugin runs at a time[3].
- Custom agents are a JSON array on the ACP providers plugin:

```bash
bb plugin config provider-acp set customAgents \
  '[{"id":"amp","displayName":"Amp","command":"amp","args":["acp"]}]'
```

  Entries take `id`, `displayName`, `command`, with optional `args`, `env`, `cwd`, `modelCli`, `reasoningCli`, `nativeReasoning`, `nativeSkillRoots`, `permissionCli`, `supportsManualCompaction`, `dialect`; changes apply without a restart[3].

- MCP: no MCP configuration section exists in the configuration doc; whether bb surfaces MCP to its agents is unverified.

## 5. Plugins and extensions

Plugins are on by default; state lives under `<dataDir>/plugins/`[3]. Marketplaces are managed with `bb marketplace add|list|refresh|remove` (catalogs stored in bb's database, not on disk); the reserved `bb-community` marketplace is served from `BB_MARKETPLACE_URL`, default `https://getbb.app/marketplace/v2/marketplace.json`[3]. Bundled official plugins (GitHub, Docs, Memory, Tasks) ship with the app and cannot be removed[3]. Per-plugin config goes through `bb plugin config <id> set ...` (push notifications, provider options like `disable1MContext`, `chromeEnabled`, `sandboxEnabled` for Claude Code)[3]. Further commands: `bb storage retention ...`, `bb keep-awake ...`, `bb concurrency-limit ...`, `bb pool ...`[3].

## 6. Multi-instance wrappers

`BB_DATA_DIR` relocates the whole data root per instance, and `BB_SERVER_PORT`/`BB_HOST_DAEMON_PORT` avoid collisions when two run side by side[3]. Provider credentials are per-instance through `~/.bb/env.json` under each data dir, while the wrapped CLIs keep their own config trees (Claude Code, Codex, ACP agents), so credential isolation composes with those harnesses' own relocation vars.

## 7. Not verified upstream

- MCP support (absent from the configuration doc; nothing fetched says yes or no).
- A version-probe command.
- Windows x64 and Linux beyond "alpha" status.
- The default `<dataDir>` when `BB_DATA_DIR` is unset beyond the `~/.bb/` root the config files imply.

## Sources

- [1] https://github.com/get-bb/bb: README (positioning, install, telemetry, nightly, platform status)
- [2] https://www.sawyerhood.com/blog/an-agentic-ide-that-builds-itself: origin story, repo and site links
- [3] https://raw.githubusercontent.com/get-bb/bb/main/docs/configuration.md: config files, env vars, ACP agents, plugins, marketplaces, precedence
- [4] https://registry.npmjs.org/bb-app: package 0.46.0
