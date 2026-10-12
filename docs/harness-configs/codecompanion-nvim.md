# codecompanion.nvim — Configuration Reference

**Compiled:** 2026-10-12 against the olimorris/codecompanion.nvim README and the codecompanion.olimorris.dev docs (MCP configuration page). codecompanion.nvim is a Neovim AI coding assistant (Apache-2.0): chat buffer, inline transformations, agents and tools, workflows, code review, an action palette, and a prompt library. Dependencies are nvim-lua/plenary.nvim and nvim-treesitter/nvim-treesitter; a `minimal.lua` ships for troubleshooting via `nvim --clean -u minimal.lua`[1].

---

## 1. Install, executable, version

A Lua plugin, not an executable. lazy.nvim form from the README[1]:

```lua
{
  "olimorris/codecompanion.nvim",
  dependencies = { "nvim-lua/plenary.nvim", "nvim-treesitter/nvim-treesitter" },
  opts = { opts = { log_level = "DEBUG" } },
}
```

Package-manager-agnostic use is `require("codecompanion").setup({...})`[1]. Diagnostics: `:checkhealth codecompanion`; `log_level = "TRACE"` for verbose logs[1]. No version-probe command is documented.

## 2. Config files

None as files: configuration is the Lua `setup()` table only. The docs' configuration pages cover adapters, prompt library, and MCP, all as Lua keys[1][2]. A `.codecompanion` directory exists in the plugin's own repo, but no page documents loading user config from a file, so treat file-based config as absent. Project-local state (chat buffers, saved chats) lives under Neovim's data directories.

## 3. Adapters and environment variables

Adapters: Anthropic, DeepSeek, Google Gemini, GitHub Copilot, GitHub Models, Kimi, Mistral, Novita, Ollama, OpenAI, Azure OpenAI, OpenRouter, HuggingFace, and xAI, plus community adapters and bring-your-own[1]. Adapter API keys are environment variables per adapter (the standard `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` pattern; values inside config can run shell commands, e.g. 1Password CLI via a `cmd:` prefix[2]). The README does not enumerate the variable names; the adapter pages on the docs site hold them.

## 4. MCP

Native MCP support, configured under the top-level `mcp` key in `setup()`[2]:

```lua
require("codecompanion").setup({
  mcp = {
    servers = {
      ["tavily-mcp"] = {
        cmd = { "npx", "-y", "tavily-mcp@latest" },
        env = { TAVILY_API_KEY = "cmd:op read op://personal/Tavily_API/credential --no-newline" },
      },
    },
    opts = { default_servers = { "sequential-thinking" } },
  },
})
```

Each server is a table with `cmd` (argv list), `env`, and optional `roots`; the value may be a function returning the table, evaluated once when the server is first needed[2]. Servers in `opts.default_servers` start automatically with the chat buffer; others start on demand via the `/mcp` slash command, and prompt-library items can name `mcp_servers` explicitly[2]. Protocol version 2025-11-25; stdio transport only; tools are supported (Text Content only) along with prompts, cancellation, roots (off by default), and pagination; not supported: Streamable HTTP, progress, tasks, sampling, elicitation, completion, resources, and tool-list-changed notifications[3].

## 5. ACP

Built-in Agent Client Protocol support lets Neovim drive external agents such as Claude Code, Codex, Gemini CLI, Goose, Copilot CLI, Cursor CLI, Kiro, and OpenCode through codecompanion[1][4].

## 6. Multi-instance wrappers

Isolation is Neovim-config isolation: the plugin's config lives in your init files, so separate `NVIM_APPNAME`/`XDG_CONFIG_HOME` roots give fully separate instances, and adapter env vars split identities per process. No plugin-specific relocation knob exists.

## 7. Not verified upstream

- The full adapter env-var name list (docs site has per-adapter pages; not all fetched).
- Where chat history is persisted on disk.
- A version-probe command.

## Sources

- [1] https://github.com/olimorris/codecompanion.nvim: README (setup forms, dependencies, adapters, ACP, checkhealth)
- [2] https://codecompanion.olimorris.dev/configuration/mcp.md: MCP setup shape, default_servers, /mcp command, cmd: env values
- [3] https://codecompanion.olimorris.dev/model-context-protocol: protocol version, supported and unsupported features
- [4] https://agentclientprotocol.com: ACP standard reference
