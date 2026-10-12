# hax — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/OleksandrChekhovskyi/hax](https://github.com/OleksandrChekhovskyi/hax) (MIT, C) and its docs/configuration.md[1][2]. hax is a minimalist terminal-native coding agent written in C: a single native binary, a few MB of memory, local models as first-class citizens, and a deliberate list of omissions (no MCP marketplaces, no plugin runtime, no IDE panels, no per-command permission prompts; composition happens via subprocesses)[1]. Latest release v0.6.0; pushed 2026-10-09: alive.

---

## 1. Install, executable, version

```sh
brew install oleksandrchekhovskyi/hax/hax       # macOS or Linux
# or the prebuilt static binary (x86_64/aarch64) from the latest release
make                                          # from source: shared-lib build
```

Linux, macOS, FreeBSD, OpenBSD; Windows via WSL. The AUR carries `hax`. Build deps: C compiler, libcurl, jansson, meson, ninja, pkg-config, plus fzf for `@file` completion when available[1].

## 2. Config files

hax follows XDG Base Directory specs; `XDG_CONFIG_HOME`, `XDG_STATE_HOME`, and `XDG_CACHE_HOME` move the corresponding roots[2]:

| Purpose | Default path |
|---|---|
| User configuration | `~/.config/hax/config.json` |
| Remembered interactive selections | `~/.local/state/hax/state.json` |
| Sessions and prompt history | `~/.local/state/hax/sessions/<encoded-cwd>/` |
| Model metadata cache | `~/.cache/hax/catalog.json` |

`config.json` is plain JSON (no comments, no trailing commas; flat dotted keys accepted for scalars). `state.json` is machine-local picker state written by hax; do not edit or version it. Resolution for an ordinary setting, first defined value wins[2]:

```text
current-process override → resumed conversation → environment → state.json → config.json → default
```

Presets are named provider/model/effort stances, optionally promoted to delegation roles by adding a description and instructions; apply via `hax <name>`, `--preset`, `HAX_PRESET`, or `/preset`[2]. `/config` shows every registered setting's resolved value and source, and can set process overrides (`/config theme light`)[2].

## 3. Providers and environment

Providers[1]: OpenAI (+compatible), Anthropic (+compatible), Codex via a ChatGPT subscription (`/login` inside hax), OpenRouter, OpenCode Zen/Go, DeepSeek, llama.cpp, Ollama, custom endpoints. Keys are environment variables: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `OPENROUTER_API_KEY`, `OPENCODE_API_KEY`, `DEEPSEEK_API_KEY`; llama.cpp needs `llama-server -m model.gguf` running, then `hax --provider llama.cpp` with auto-discovery and no provider config block[1]. `HAX_*` environment variables override saved interactive choices[2].

## 4. Headless

```sh
hax -p "list TODOs"          # one-shot; clean stdout, resume hints on stderr
printf "explain x" | hax -p  # prompt from stdin
hax -c                       # continue latest session for this directory
hax --resume=ID -p "next"    # resume a specific session one-shot
hax --json                   # JSON session stream (docs/sessions.md)
```

Resumed conversations restore their own provider, model, effort, and preset unless a CLI flag overrides[1][2].

## 5. MCP, skills, plugins

Deliberately absent: no MCP, no plugin runtime, no marketplace. The philosophy doc covers each omission and the subprocess-composition pattern that replaces it[1].

## 6. Not verified upstream

- The complete `config.json` key reference (docs/configuration.md documents the format, resolution, and presets; the full settings table lives in the same file's reference section, not fully extracted here).
- The `hax --json` event schema (docs/sessions.md was not fetched).
- A config-dir env var beyond the XDG trio (none is documented; `~/.config/hax` itself is not directly overridable by an `HAX_*` var per the fetched pages).

## Sources

- [1] https://raw.githubusercontent.com/OleksandrChekhovskyi/hax/master/README.md: positioning, install, providers, quick start, deliberate omissions
- [2] https://raw.githubusercontent.com/OleksandrChekhovskyi/hax/master/docs/configuration.md: XDG paths, resolution order, presets, HAX_* variables, /config
- [3] https://github.com/OleksandrChekhovskyi/hax/releases/tag/v0.6.0: latest release; repo pushed 2026-10-09
- [4] https://github.com/Qumulo/llm-agents/tree/main/packages/hax: nix index entry (owner OleksandrChekhovskyi, v0.6.0)
