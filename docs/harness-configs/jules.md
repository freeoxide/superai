# Jules CLI (`@google/jules`) — Configuration Reference

**Compiled:** 2026-10-12 against the official CLI reference (jules.google/docs/cli/reference), the npm package `@google/jules` (0.1.42 latest), and its published launcher[1][2]. Jules is Google's asynchronous coding agent: tasks run on Google cloud VMs against GitHub repos, not in your local shell. "Jules Tools" is the CLI companion announced 2025-10-02 alongside a preview REST API[3][4]. The agent is Gemini-backed; the docs site itself sits behind Google sign-in, so several details below stop at what the public pages and the package state.

---

## 1. Install, executable, version

```bash
npm install -g @google/jules      # pnpm works too
jules version
jules --help
```

The npm package is a launcher that fetches a Go binary for the platform (darwin/linux/windows, amd64/arm64 per its arch mapping) into the package bin directory[2]. There is no public source repo for the CLI; GitHub hosts community clients and a Gemini CLI extension instead[1][2].

## 2. Commands

From the README and CLI reference[1][2]:

| Command | Purpose |
|---|---|
| `jules` | Interactive TUI dashboard with session view, diff viewer, guided session creation |
| `jules login` / `jules logout` | Google account browser OAuth |
| `jules new "<prompt>"` (`remote new`) | Create a session; `--repo torvalds/linux` or `.` for cwd repo; `--parallel <n>` for parallel sessions on one task |
| `jules remote list --repo` / `--session` | List connected repos / sessions |
| `jules remote pull --session <id>` | Pull results and code changes from a completed session |
| `jules completion <shell>` | Shell autocompletion |
| `jules version` | Installed CLI version |

Global flag: `--theme dark|light` (default dark)[1]. Piping works (`cat TODO.md | while read line; do jules new "$line"; done`), which is the documented batch pattern[2].

## 3. Config and state

The reference documents no config files, no environment variables, and no local settings surface[1]. Login state is stored by the binary at an undocumented location. Because work happens remotely, there is no local sandbox, MCP client, or skills surface: the CLI is a session orchestrator, not a local harness. Treat "no local config" as unverified-absent rather than vendor-confirmed: the deeper docs are sign-in gated.

## 4. Auth, limits, API

- Auth is Google browser OAuth only; no API-key or service-account path appears in the reference[1].
- Usage limits are plan-based (free tier reported by third parties at 15 tasks per rolling 24 hours, 3 concurrent; higher with Google AI Pro/Ultra). Verify current numbers on the jules.google plans page before relying on them; they were not on a fetched official page[3].
- The Jules REST API exists in preview at developers.google.com/jules/api with an OpenAPI description; community TypeScript clients and MCP servers are built on it[3][4].

## 5. Wrapper relevance

Isolation for multi-instance use reduces to separate Google accounts (`jules login` per environment); there is no config-dir env var to split, and sessions live server-side. An adapter's realistic surface is install, version probe, login state check, and scripted `remote new`/`remote pull`.

## 6. Not verified upstream

- Local credential/config storage path (undocumented; docs gated).
- Officially stated usage limits (third-party figures only).
- MCP support in the CLI: absent from the reference; the MCP servers that exist are third-party bridges exposing the Jules API to other agents[3].
- Windows binary completeness (the launcher maps win32, but the reference lists no Windows specifics).

## Sources

- [1] https://jules.google/docs/cli/reference: commands, flags, OAuth flow
- [2] `@google/jules` npm package 0.1.42 (https://registry.npmjs.org/@google/jules): README, help text, Go-binary launcher mappings
- [3] Google Developers Blog, "Meet Jules Tools: A Command Line Companion" (2025-10-02): announcement; third-party coverage of limits (therundown.ai, agentcode.ai)
- [4] https://developers.google.com/jules/api: Jules REST API preview
