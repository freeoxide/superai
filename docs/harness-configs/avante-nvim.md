# avante.nvim — Configuration Reference

**Compiled:** 2026-10-12 against the avante.nvim README (repo now avante-corp/avante.nvim, 18.2k stars, Apache-2.0, active; the old yetone/avante.nvim URL 301-redirects there) and the mcphub.nvim extension docs. avante.nvim is a Neovim plugin that emulates the Cursor AI IDE: AI-driven edits applied straight to source files, plus chat, suggestion, and agentic modes. It requires Neovim 0.11.0+ and builds with `make` (a PowerShell script on Windows)[1].

---

## 1. Install, executable, version

There is no executable; it is a Lua plugin. lazy.nvim spec from the README[1]:

```lua
{
  "avante-corp/avante.nvim",
  event = "VeryLazy",
  build = "make",   -- PowerShell script on Windows
  opts = {},
}
```

Also installable via native `vim.pack.add`, rocks.nvim, vim-plug, mini.deps, Home Manager, and Nixvim[1]. No version-probe command is documented; the pinned commit in your plugin spec is the version.

## 2. Config files

There is no separate config file. Configuration is Lua, in one of two forms, with `setup` options winning over the global[1]:

```lua
vim.g.avante = { ... }                    -- global-variable form
require("avante").setup({ ... })          -- setup form; defaults in lua/avante/config.lua
```

File locations the plugin reads[1]:

| Path | Purpose |
|---|---|
| `<project>/avante.md` | Project instructions (relocatable via `instructions_file`) |
| `*.planning.avanterules` etc. at project root | Per-mode rules files |
| `rules = { project_dir, global_dir }` | Rules directories |

Because everything is Lua, "editing config" means editing the plugin spec in your Neovim config; multi-instance isolation means separate Neovim config roots (e.g. `NVIM_APPNAME` or `XDG_CONFIG_HOME` swaps), which the fetched pages do not discuss.

## 3. Providers and environment variables

Default provider is `claude`; also openai, azure, gemini, cohere, copilot, moonshot, bedrock, morph (Fast Apply), ollama, and llama-cpp, plus agents reached over ACP (codex, gemini, claude-code) under `acp_providers`[1]. API keys are environment variables: `AVANTE_`-prefixed per-provider keys (e.g. `AVANTE_ANTHROPIC_API_KEY`) or the legacy global names (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, ...). Web search wants `TAVILY_API_KEY`, `KAGI_API_KEY`, or `BRAVE_API_KEY`; Fast Apply wants `MORPH_API_KEY`[1].

## 4. MCP

MCP goes through mcphub.nvim: avante's docs point at the mcphub Avante extension, so the server definitions live in mcphub.nvim's config, not in avante's own setup[1][2].

## 5. Not verified upstream

- Whether an `mcp_servers` key exists in avante's own setup (the README documents the mcphub path; no native key was found).
- ACP configuration details beyond the `acp_providers` key's existence.
- Any version or health-check command.

## Sources

- [1] https://github.com/avante-corp/avante.nvim: README (config forms, providers, env vars, rules files, ACP, MCP via mcphub, install options, Neovim 0.11 requirement)
- [2] https://ravitemer.github.io/mcphub.nvim/extensions/avante.html: mcphub Avante extension (linked from avante README)
