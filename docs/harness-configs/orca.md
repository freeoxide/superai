# Orca (Stably AI) — Configuration Reference

**Compiled:** 2026-10-12 against onorca.dev (home and settings reference), the stablyai/orca GitHub repo (90k stars, MIT, active), and search-verified coverage. Orca is an "agent development environment" (ADE): a desktop app built for running a fleet of parallel coding agents, each in its own git worktree, with terminals, diffs, an embedded Chromium browser, and a companion CLI. Mobile apps (iOS App Store, Android APK) and SSH remote worktrees extend it off the desktop[1][2]. It ships no agent of its own: 27 CLIs are supported (Claude Code, Codex, OpenCode, Grok, Gemini, Cursor, GitHub Copilot, Amp, OpenClaude, Antigravity, Pi, oh-my-pi, Hermes Agent, Goose, Auggie, Charm, Cline, Codebuff, Command Code, Continue, Droid, Kilocode, Kimi, Kiro, Mistral Vibe, Qwen Code, Rovo Dev) plus "any other CLI agent", all using your own subscriptions[1][2]. Mechanically it sits with the orchestrators in [orchestrators.md](orchestrators.md), but it edits agent config in more places than they do, which is what matters here.

---

## 1. Install, executable, version

Desktop downloads from onorca.dev/download (macOS Apple Silicon and Intel, Windows, Linux); iOS via App Store (id 6766130217); Android as an APK from GitHub releases[1][2]. The CLI ships with the app (`orca worktree create|snapshot|click|fill`, `orca agent hooks on|off|status`)[1][2]. No version-probe command is documented; the app updates itself.

## 2. Config files

| Path | Purpose | Source |
|---|---|---|
| `~/.orca/keybindings.json` | Keybinding overrides (existing installs keep customized tab-navigation bindings here) | [3] |
| `<repo>/orca.yaml` | Repo-checked-in settings; documented key `worktree.sharedDirectories`, plus per-repo base ref, hooks, auto-run commands on worktree create, icons and badge colors, Source Control AI overrides | [3] |
| `<repo>/.worktreeinclude` | Files copied into every worktree (Conductor's pattern, also here) | [3] |

Everything else is settings-UI state (Cmd-, searchable panes), stored by the app at an undocumented location[3].

## 3. Agent launch and credential configuration

- Per-agent launch defaults let you override the binary path and name and edit the default command, arguments, and environment; structured native chats reuse the saved command and arguments[3].
- Detected agents can be enabled or disabled so launch menus show only the CLIs you want[3].
- Agent permissions: Yolo vs Manual; folder-trust bypass is on by default for agents Orca launches (orchestration workers, automations, phone-started agents)[3].
- Account lists exist for Claude, Codex, OpenCode, Devin, Antigravity, and the GLM Coding Plan[3].
- `ORCA_BITBUCKET_*` environment variables supply Bitbucket credentials, take precedence over saved credentials, and hide the connect buttons; on Remote Orca Servers they must be set on the server[3].
- An OpenAI API key (Voice pane) powers cloud voice models; an OpenCode Go session cookie can be pasted for OpenCode accounts[3].

## 4. MCP, skills, hooks, plugins

- MCP servers are configured under Settings → Integrations[3]. The mechanics beyond that location are on a dedicated skills-and-MCP page that was not fetched; treat the write path as unverified.
- Hooks: per-agent startup hooks (Agents pane, toggleable without restart) and per-repo hooks (Repository pane); CLI control via `orca agent hooks on|off|status`[3].
- Skills: a skills registry with background freshness updates, managed from an Agents-pane dialog[3].
- Plugins: experimental system with git-based marketplaces; plugin workers always run on the local machine while SSH workspace actions route through Orca[3].

## 5. Remote and multi-host

Remote Orca Servers pair with the desktop, which can advertise itself as a server with revocable access links; machine names identify paired mobile hosts; a default runtime can be picked for server-routed projects and terminals[3]. SSH targets support worktrees, passphrases, proxy/jump hosts, OpenSSH multiplexing (default on), and per-host CLI control permission; managed servers let Orca install, update, roll back, recover, and stop its own server on an SSH host[3]. Codex terminals get their own server per terminal by default[3].

## 6. Multi-instance wrappers

Orca is the wrapper: per-agent binary path and env overrides are its native mechanism for routing the same CLI at different providers[3]. For isolating Orca itself, no config-dir env var or relocation knob is documented; the only file-path anchors are `~/.orca/keybindings.json` and repo-level `orca.yaml`/`.worktreeinclude`.

## 7. Not verified upstream

- Where the app stores its settings state on disk.
- MCP configuration mechanics under Settings → Integrations (dedicated page not fetched).
- A version-probe command.
- Worktree root default path (worktrees per agent are documented; the parent directory is not).

## Sources

- [1] https://www.onorca.dev: product overview, platforms, download links, 27-agent list, CLI, mobile apps
- [2] https://github.com/stablyai/orca: MIT license, activity, description
- [3] https://www.onorca.dev/docs/settings: settings reference (keybindings.json, orca.yaml, agent launch defaults, permissions, env vars, hooks, skills, plugins, remote servers, SSH)
