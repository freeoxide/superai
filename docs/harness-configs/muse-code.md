# Muse Code (Meta) — Configuration Reference

**Compiled:** 2026-10-12 against [dev.meta.ai/docs](https://dev.meta.ai/docs/muse-code). Muse Code is Meta's coding agent for the terminal and CI, built for the Muse Spark models and powered by the Meta Model API (`https://api.meta.ai/v1`)[1][2]. First released 2026-08-05; press coverage and the free-tier framing call the current product a beta, and the docs themselves carry no status label[3][4]. Windows, macOS, and Linux ship from one codebase; on Windows, PowerShell replaces Bash, the sandbox may raise a UAC prompt, and voice input plus session messaging are unavailable[1].

---

## 1. Install, executable, version

```bash
curl -fsSL https://dev.meta.ai/install.sh | sh      # macOS / Linux
irm https://dev.meta.ai/install.ps1 | iex            # Windows PowerShell
muse --version
```

The executable is `muse`[1]. Subcommands: `muse` (interactive), `muse exec "<prompt>"` (headless), `muse serve` (protocol server), `muse schema` (print settings schema), `muse config validate|status`, `muse auth set`, `muse logout`, `muse skills …`, `muse mcp login|logout <server>`, `muse export`[1][5][6]. A TypeScript SDK (`npm install @muse-code/sdk`) wraps the same session protocol[1].

## 2. Config files

| Path | Purpose | Source |
|---|---|---|
| `~/.config/muse/settings.json` | Global settings, JSON, requires `"schema_version": 1`. Missing file is fine (defaults); wrong schema_version fails startup | [5] |
| `<repo>/AGENTS.md` (fallbacks `CLAUDE.md`, `.agents/AGENTS.md`, `.claude/CLAUDE.md`) | Project instructions; first existing file wins per directory, walking up to the nearest `.git`; project rules load only in trusted workspaces | [5] |
| `<repo>/.muse/hooks.json` | Project hooks; user hooks live in the settings file, managed hooks at `managed_hooks_path` | [6] |
| `<repo>/.agents/memory/` | Committed project memory (`MEMORY.md` index plus per-topic files); personal memory stays outside the repo | [5] |

Documented settings keys: model defaults, `tui.terminal_background`, `permissions.default_profile`, an `mcp_servers` block, hooks plus `managed_hooks_path`, `runtime_capabilities`, telemetry, `agents.execution_capacity` (1-64, default 8 per tree)[5][6]. Enterprise admins can validate managed planes with `muse config validate --plane <defaults|policy> --file <path>`[5].

No config-dir override (`MUSE_HOME`, `MUSE_CONFIG_DIR`, or similar) appears anywhere in the docs. Multi-instance isolation has no documented lever beyond `HOME` relocation; treat that as unverified until Meta documents one.

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `META_API_KEY` | API key auth for CI; also storable via `muse auth set`. Priority: env var, stored key, stored browser session | [7] |
| `MUSE_TRANSPORT_TRACE=1` | Print raw provider requests/responses to stderr, credentials scrubbed | [5] |
| `MUSE_SESSION_ID` | Injected into stdio MCP server environments | [6] |

Auth on first run is browser sign-in or a pasted API key; `/login` reopens the choice[1][7]. Meta Managed Account users cannot use browser sign-in and must use an API key[7]. Billing is usage-based per token, with flat monthly subscriptions available[7]. There is no BYO-endpoint surface: the agent talks only to the Meta Model API.

## 4. MCP

Configured in the `mcp_servers` block of `settings.json`, stdio or streamable_http[6]:

```json
{ "mcp_servers": {
  "my-tools": { "transport": "stdio", "command": "my-mcp-server", "args": [] }
} }
```

- Every server also takes `enabled` and `mode` (`required` default; `optional` servers that fail are skipped with a warning). A non-default `framing` on stdio or any `framing` on streamable_http fails validation.
- `${VAR}` interpolation is supported; `muse mcp login <server>` handles OAuth 2.1, and sign-ins from other terminals are picked up without a restart. Disable dynamic client registration with `"mcp_oauth_dynamic_client_registration": false`.
- `/mcp` lists connected servers; read-only-declared tools skip approval under on-request approvals.

superai can write `mcp_servers` entries by editing `~/.config/muse/settings.json` while preserving unmodelled keys and `schema_version`.

## 5. Skills

Load sources[6]: built-in; user (`$XDG_CONFIG_HOME/muse/skills`, `~/.agents/skills`, plus discovered `~/.claude/skills` and `$CODEX_HOME/skills` falling back to `~/.codex/skills`); project (`<repo>/.agents/skills/<skill-id>/SKILL.md`, `.codex/skills`, `.claude/skills`); plugin bundles. Commands: `muse skills list|inspect|enable|install|validate|import --from claude|codex`. Built-in slash skills include `/plan`, `/grill`, `/taste`, `/threejs`, and `/migrate` (imports Claude Code/Codex memory and MCP servers).

## 6. Headless / CI

`muse exec` flags: `--prompt-file <path>`, `--json`, `--disable-approval`, `--yolo`, `--max-model-steps <n>`, `--session-id <uuid>`, `--allow-workspace-switch`; `muse export --session <uuid> --out run.json`[6]. Exit codes: 0 turn complete, 1 failure or cancel, 2 usage error, 130/143 on SIGINT/SIGTERM[6]. Sandbox is on by default (`--sandbox-network <mode>`, `--disable-sandbox`), approvals default on (`--approval-mode`, `--approval-judge on|off`)[5].

## 7. Not verified upstream

- Exact on-disk location of stored credentials (docs say "stores the credential for you"; no path published)[7].
- Any config-dir relocation env var. Absence is per the current docs set, not a verified-absent statement from Meta.
- A published `muse --help` transcript; flags above come from the configuration and extending pages only.

## Sources

- [1] https://dev.meta.ai/docs/muse-code — Muse Code overview (platforms, install, auth, subcommands, SDK)
- [2] https://dev.meta.ai/docs/overview/ — Meta Model API overview (base URL, Muse Spark models)
- [3] https://www.cdomagazine.tech/ — Meta launches Muse Code beta (2026-08-10 coverage)
- [4] https://www.infoworld.com/ — Meta launches Muse Code for complex software work (2026-08-06)
- [5] https://dev.meta.ai/docs/muse-code/configuration — settings.json, AGENTS.md lookup, flags, memory
- [6] https://dev.meta.ai/docs/muse-code/extending — MCP block, skills, hooks, subagents, muse exec
- [7] https://dev.meta.ai/docs/muse-code/auth — META_API_KEY, credential priority, billing
