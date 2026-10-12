# Kilo CLI (`@kilocode/cli`) — Configuration Reference

**Compiled:** 2026-10-12 against [kilo.ai/docs/code-with-ai/platforms/cli](https://kilo.ai/docs/code-with-ai/platforms/cli) and the CLI comparison page. Kilo CLI is the terminal agent of the Kilo Code platform: a fork of OpenCode maintained in the Kilo-Org/kilocode repository, sharing its config schema while adding the Kilo Gateway, teams, and remote control[1][2]. The VS Code/JetBrains extension surface is covered in [kilo-code.md](kilo-code.md); both read the same `kilo.jsonc` files[1]. Docs apply to Kilo 1.0+[1].

---

## 1. Install, executable, version

```bash
npm install -g @kilocode/cli
kilo --version        # -v
kilo upgrade          # or npm update -g @kilocode/cli; kilo uninstall removes it
```

The install page also offers curl, pnpm, bun, brew, and paru/AUR tabs (exact commands render per tab on the page; npm is the documented default)[2]. CPUs without AVX use `-baseline` release binaries (kilo-linux-x64-baseline.tar.gz, kilo-darwin-x64-baseline.zip, kilo-windows-x64-baseline.zip): extract and run `kilo` directly[1]. First-run provider setup is `/connect`; `kilo auth` manages credentials afterwards[1].

## 2. Config files

| Path | Purpose | Source |
|---|---|---|
| `~/.config/kilo/kilo.jsonc` | Global config (legacy `kilo.json`, `opencode.json[c]` deep-merged) | [1] |
| `./kilo.jsonc` or `./.kilo/kilo.jsonc` | Project config (`.kilo/` wins if both exist; legacy `.kilocode/` also read; `.opencode/` dirs no longer consulted) | [1] |
| `~/.config/kilo/tui.jsonc` / `.kilo/tui.json` | TUI settings (keybinds) | [1] |

Schema `$schema: "https://app.kilo.ai/config.json"`; keys shared with OpenCode pass through (`model`, `provider.<id>.options.*`, `mcp`, `permission`, `agent`, `command`) plus Kilo's `top`/`agents`/`experimental` overlays[3]. `{env:VAR}` interpolation resolves only in trusted config: global files, `KILO_CONFIG`/`KILO_CONFIG_CONTENT`, or org-managed config, not repo-committed project config[1]. Remote mode toggle persists as `"remote_control": true` in config[1].

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `KILO_PROVIDER` | Active provider id override | [1] |
| `KILOCODE_<FIELD>` | Kilocode (gateway) provider fields (`KILOCODE_MODEL` → `kilocodeModel`) | [1] |
| `KILO_<FIELD>` | Other providers' fields (`KILO_API_KEY` → `apiKey`) | [1] |
| `KILO_ORG_ID` | Organization selection, non-interactive/CI | [1] |
| `KILO_CONFIG` / `KILO_CONFIG_CONTENT` | Config file path / inline config content (trusted; highest config precedence) | [1] |
| `KILO_DISABLE_MOUSE=1` | Terminal link diagnostic | [1] |
| `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_EXPORTER_OTLP_HEADERS`, `OTEL_RESOURCE_ATTRIBUTES` | OpenTelemetry (off via `experimental.openTelemetry = false`) | [1] |

No dedicated config-dir override exists; the global path derives from the OS config home, so `HOME`/`XDG_CONFIG_HOME` relocation or `KILO_CONFIG` content injection is the isolation lever[1].

## 4. CLI commands

`kilo` (TUI in a directory), `kilo run [message..]` (one-shot; `--auto` skips all approval prompts and auto-rejects non-approved ops; exit codes 0 success, 124 timeout, 1 error), `kilo --continue`/`-c` (resume latest; not combinable with `--auto` or a prompt arg), `kilo serve` (headless server), `kilo attach <url>`, `kilo acp` (ACP server), `kilo daemon`, `kilo auth`, `kilo agent`, `kilo mcp`, `kilo models [provider]`, `kilo profile`, `kilo stats` (token usage/cost), `kilo export [sessionID]` / `kilo import <file>`, `kilo github`, `kilo pr`, `kilo session`, `kilo remote`, `kilo db`, `kilo config`, `kilo plugin <module>`, `kilo debug config` (inspect merged config), `kilo completion`[1]. Global flags: `--print-logs`, `--log-level DEBUG|INFO|WARN|ERROR`[1].

## 5. MCP, providers, skills

- MCP: servers live under the top-level `mcp` key in `kilo.jsonc` (local stdio `command` arrays or remote `url` + `headers`, `enabled`, `timeout`, OAuth automatic for remote unless `"oauth": false`); managed via `kilo mcp`[1][3].
- Providers: Kilo Gateway (500+ hosted models, Kilo Pass $19/$49/$199 per month or pay-as-you-go credits), OpenRouter, OpenAI, Anthropic, Ollama, LM Studio, and any OpenAI-compatible or Anthropic-Messages endpoint via custom providers[2][3].
- Session interop: `/resume-claude <uuid>` and `/resume-codex <uuid>` import sessions from `~/.claude/projects/` and `~/.codex/sessions/` into an empty Kilo session[1].

## 6. Multi-instance wrappers

Documented primitives: `KILO_CONFIG_CONTENT` inline config per process (trusted precedence), `KILO_PROVIDER`/`KILO_*` field overrides per provider instance, `KILO_ORG_ID` for org routing in scripts, and `HOME`/`XDG_CONFIG_HOME` relocation for a fully separate global config + auth state. The wrapper sketch in [kilo-code.md](kilo-code.md) §4.3 applies; `kilo run --auto` is the CI entry (`kilo run "Implement the new feature" --auto`)[1].

## 7. Not verified upstream

- Exact brew tap name and AUR package name (tabs on the install page did not render commands into the fetched text).
- Where gateway credentials are stored on disk (`kilo auth` manages them; no path published on the fetched pages).
- `kilo mcp` subcommand surface beyond its existence.

## Sources

- [1] https://kilo.ai/docs/code-with-ai/platforms/cli — commands, flags, env vars, config files, exit codes, resume interop
- [2] https://kilo.ai/cli/opencode — install tabs, platform surfaces, Kilo Pass pricing, OpenCode relationship
- [3] https://kilo.ai/docs/getting-started/settings and https://kilo.ai/docs/contributing/architecture/config-schema — config files, schema, mcp key shape (as cited in kilo-code.md, re-checked 2026-10-12)
