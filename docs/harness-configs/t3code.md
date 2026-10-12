# T3 Code (`t3`) — Configuration Reference

**Compiled:** 2026-10-12 against the pingdotgg/t3code README and the in-repo user docs (install, project-settings, permission-modes), plus the GitHub API (27k stars, MIT, active). T3 Code calls itself an "agent harness control surface": an Electron desktop app, a web app, and mobile companions that drive AI coding agents already installed on your machine. It is not an agent itself; it launches and supervises CLIs you have authenticated yourself[1][2]. Supported providers: Codex, Claude Code, Cursor, Grok Build, OpenCode, and Google Antigravity, plus ACP registry agents and Muse Code as seen in the permission docs[2][4]. This is the same genus as the orchestrators in [orchestrators.md](orchestrators.md), but with a CLI server at its core rather than a single GUI.

---

## 1. Install, executable, version

```bash
curl -fsSL https://t3.codes/install.sh | sh     # macOS / Linux; binary lands in ~/.local/bin
irm https://t3.codes/install.ps1 | iex           # Windows PowerShell
npx t3@latest                                    # one-off run without installing
brew install --cask t3-code                      # macOS desktop
winget install T3Tools.T3Code                    # Windows desktop (also scoop, .deb, AUR)
```

Commands: `t3` starts the server and opens the local web app; `t3 service install` keeps it running in the background on macOS/Linux; `t3 update` upgrades; `t3 uninstall` removes it; `t3 --help` for the full reference[1][3]. No dedicated version command is documented. Intel Macs get no prebuilt `t3` executable and must build from source (Node 24, `vp run build:desktop`)[3]. Mobile apps ship on the App Store and Google Play; the web app also runs at app.t3.codes[1][2].

## 2. Config and settings

There is no user-facing JSON settings file. Settings split three ways[4]:

| Layer | Where | Notes |
|---|---|---|
| Device preferences | On the device only | Appearance, confirmations, browser profiles; ignore project/environment selection |
| Environment settings | The t3 server's own store, per environment | Most settings; editable from the app's settings UI |
| Project overrides | Per project, layered over the environment | Resettable back to inherit |
| `t3.json` | Committed to the repo | Declares repo-level settings, e.g. the workspace for new threads and `worktreeSubmodules`; its actions can be imported into a project's actions list |

Resolution order: project override, then environment setting, then `t3.json`, then the built-in default[4]. Worktree location is per machine, not per project[4]. `--base-dir` runs an independent server with its own root, which is the closest thing to a relocation knob[3].

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `T3CODE_CHANNEL` | `nightly` switches to the nightly update channel | [3] |
| `T3CODE_VERSION` | Pin an exact release | [3] |

The repo's `.env.example` targets development of t3code itself, not user configuration.

## 4. Permission modes

Four modes for agent threads: Supervised (approve commands and edits), Auto-accept edits, Auto (provider-driven approval; works on Codex, Claude, Cursor, and Grok), and Full access (no prompts). The initial default is Full access, changeable under Settings → General → New threads → Permissions, with project overrides[5]. Grok lacks Auto-accept edits; Muse Code supports only Supervised and Full access; ACP registry agents run their own tools in their own mode while t3 handles their approval requests; Antigravity may still send native approval requests even in Full access[5].

## 5. Providers and MCP

Each provider must be installed and authenticated before t3 will drive it: `codex login`, `claude auth login`, Cursor's `agent login`, `grok login`, `opencode auth login`; Antigravity needs no CLI (enable in settings, install, sign in with Google)[2]. The fetched pages do not document an MCP surface of t3's own; MCP servers configured for the underlying CLIs apply as those CLIs see them.

## 6. Multi-instance wrappers

`--base-dir` gives an independent server root per instance[3]. Agents are externally owned (their own config trees, e.g. `CODEX_HOME`, `CLAUDE_CONFIG_DIR`), so provider isolation happens in the wrapped CLIs, not in t3. Remote access from a phone or another machine is a documented feature[2].

## 7. Not verified upstream

- Where the t3 server stores environment settings on disk (no path documented; the settings UI is the only documented access).
- A version-probe command.
- Whether t3 exposes MCP servers to the agents it launches (nothing on the fetched pages).

## Sources

- [1] https://github.com/pingdotgg/t3code: README (positioning, providers, install matrix, remote/mobile)
- [2] https://github.com/pingdotgg/t3code: README provider section (per-CLI login requirements, web/mobile apps, remote access)
- [3] https://raw.githubusercontent.com/pingdotgg/t3code/main/docs/user/install.md: install paths, --base-dir, T3CODE_CHANNEL/T3CODE_VERSION, service install
- [4] https://raw.githubusercontent.com/pingdotgg/t3code/main/docs/user/project-settings.md: settings layers, t3.json, resolution order
- [5] https://raw.githubusercontent.com/pingdotgg/t3code/main/docs/user/permission-modes.md: the four modes, defaults, per-provider behavior
