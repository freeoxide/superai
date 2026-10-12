# Unreal Agent — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/unreallabsai/unreal-agent](https://github.com/unreallabsai/unreal-agent) (Go, v0.3.1, pushed 2026-10-06), the repo README, and the TUI client README[1][2]. Unreal Agent is an async-first agent harness from Unreal Labs: a Go library (harness/), a cloud agent runner (`cmd/unreal-agent-runner`), and a terminal client (`cmd/unreal-agent-tui`, binaries `uat` and `unreal-agent-tui`). The harness model is operation-based: tool translators validate model tool calls and emit serializable operations to an actor-runtime operation manager; sessions are append-only, forkable, and versioned[1]. User-facing surface is thin at v0.3.x; what exists is documented below.

---

## 1. Install, executable, version

```sh
brew install unreallabsai/tap/unreal-agent
unreal-agent            # same TUI as uat / unreal-agent-tui
# from source:
go -C cmd/unreal-agent-tui install .
unreal-agent-tui -help
```

## 2. Config files

One documented file: `$XDG_CONFIG_HOME/unreal-agent/preferences.json` (default `~/.config/unreal-agent/preferences.json`), written when you accept the first-launch setup dialog; it stores the provider, model, and reasoning-effort choice. `unreal-agent-tui -setup` reopens the dialog[2]. No other user-facing config file is documented; harness behavior is library/API surface, not files.

## 3. Environment variables

| Var | Purpose | Source |
|---|---|---|
| `UNREAL_HARNESS_LLM_PROVIDER` | Provider selection (same as `-provider`) | [2] |
| `UNREAL_HARNESS_LLM_MODEL` | Model selection (same as `-model`) | [2] |
| `UNREAL_HARNESS_LLM_REASONING_EFFORT` | Reasoning effort | [2] |

Precedence: explicit flags > the three env vars > saved preferences. Overrides apply to the current launch; accepting the setup dialog updates the saved choice[2].

## 4. Providers, Codex discovery

On first launch without a provider, the TUI discovers an existing Codex subscription login and offers to use it; the model selector then reads the Codex catalog from `settings.json`, including custom models (defaults offered: `gpt-6.1-sol`, high reasoning, Turbo Vision theme). `-provider openai-codex` reuses saved settings; providers without a registered default model require `-model`[2]. This reads the codex-cli harness's auth and settings, the same foreign-state pattern every-code and omp have.

## 5. Not verified upstream

- Everything below the TUI surface: the harness library's configuration (if any), the cloud runner's configuration, and operation-manager backends are undocumented outside the source.
- MCP support (not mentioned in either README).
- Skills/AGENTS.md guidance handling.
- Version probe (`-help` is documented; a `--version` flag is not).

## Sources

- [1] https://raw.githubusercontent.com/unreallabsai/unreal-agent/main/README.md: async-first harness architecture, components, versioned session invariants
- [2] https://raw.githubusercontent.com/unreallabsai/unreal-agent/main/cmd/unreal-agent-tui/README.md: brew install, preferences.json path, UNREAL_HARNESS_LLM_* vars, Codex discovery, flags
- [3] https://github.com/unreallabsai/unreal-agent/releases/tag/v0.3.1: latest release at compile time
- [4] https://github.com/Qumulo/llm-agents/tree/main/packages/unreal-agent: nix index entry (owner unreallabsai, v0.3.1)
