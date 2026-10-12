# VS Code Copilot Agent Mode — Configuration Reference

**Compiled:** 2026-10-12 against code.visualstudio.com Copilot agent-customization docs and release notes. This is the IDE surface of GitHub Copilot: agent mode inside VS Code's chat editors, where the agent edits files, runs commands, and works through MCP tools. It is a separate config surface from [copilot-cli.md](copilot-cli.md), which owns `~/.copilot`; the split mirrors cursor.md vs cursor-agent.md and kilo-code.md vs kilocode-cli.md. The two surfaces do overlap at the edges: VS Code reads the CLI's `$COPILOT_HOME/mcp-config.json` for MCP servers[1], and personal always-on instructions live in `~/.copilot/copilot-instructions.md` on the CLI's turf[2]. The overlap is read-only sharing; the settings, profiles, and workspace config below exist only in the IDE.

---

## 1. Executable, version

There is no separate binary: the agent ships inside VS Code itself. Version probe: `code --version`. Headless driving of the editor follows VS Code's own CLI (`code --user-data-dir`, `--profile`, remote tunnels)[3]; the agent has no documented standalone print mode, unlike the CLI's `copilot -p`.

## 2. Settings

Configuration lives in VS Code `settings.json` at user, workspace, or profile scope[4]:

| Key | Purpose | Source |
|---|---|---|
| `chat.agent.enabled` | Toggles agent mode availability in the Chat view (Chat > Agent: Enabled; shipped in Stable with v1.99, April 2025) | [5][6] |
| `github.copilot.chat.codeGeneration.useInstructionFiles` | Enables discovering workspace instruction files | [2] |

Agent mode needs a Copilot subscription and a recent VS Code build. Enterprise admins can control `chat.agent.enabled` through the `ChatAgentMode` policy, part of VS Code's managed AI settings[7].

## 3. Instruction files

VS Code Copilot reads, per repo[2]:

- `.github/copilot-instructions.md`, or `AGENTS.md`, as project-wide instructions; `~/.copilot/copilot-instructions.md` for personal always-on instructions in agent sessions.
- `.github/instructions/**/*.instructions.md` files, attached automatically when the `applyTo` glob in their frontmatter matches a file the agent touches (e.g. `applyTo: '**/*.py'`), or on demand when their `description` fits the task.

Both families are plain markdown in the repo (or under `~/.copilot` for the personal one), so a wrapper can write them without touching the IDE.

## 4. MCP servers

Three files plus a CLI flag, all documented[1]:

| Location | Format | Scope |
|---|---|---|
| `.vscode/mcp.json` | VS Code shape: top-level `servers` object | Workspace, committable |
| `.mcp.json` (repo root) | Portable shape: top-level `mcpServers` | Workspace, shared with other tools |
| User profile `mcp.json` (opened via `MCP: Open User Configuration`) | VS Code `servers` shape | All workspaces, per profile |
| `$COPILOT_HOME/mcp-config.json` (default `~/.copilot/mcp-config.json`) | Portable `mcpServers` | Shared with Copilot CLI |

```json
{
  "servers": {
    "github": { "type": "http", "url": "https://api.githubcopilot.com/mcp" },
    "playwright": { "command": "npx", "args": ["-y", "@microsoft/mcp-server-playwright"] }
  }
}
```

Commands: `MCP: Open User Configuration`, `MCP: Open Workspace Folder Configuration`, `MCP: Add Server` (guided), `MCP: List Servers`, `MCP: Reset Trust`. `code --add-mcp '{"name":"my-server","command":...}'` adds a server to the user profile from a shell[1]. Transports: HTTP (`"type": "http"` with `url`) and stdio (`command`/`args`); on macOS/Linux stdio servers can be sandboxed with `"sandboxEnabled": true`. Dev containers get MCP through a `customizations.vscode.mcp` block in `devcontainer.json`[1]. All of these files are user-writable JSON, which makes MCP the most wrapper-friendly part of this surface.

## 5. Multi-instance wrappers

The universal IDE pattern applies: `code --user-data-dir <dir>` gives fully separate extension and settings state[3], and VS Code profiles keep separate user-level `mcp.json` and settings within one install[4]. Workspace-level config (`.vscode/mcp.json`, `.mcp.json`, instruction files) travels with the checkout, so per-project isolation is free. There is no Copilot-specific env knob for the IDE; the IDE's own mechanisms are the whole story.

## 6. Not verified upstream

- The absolute path of the user profile `mcp.json` (docs reach it via the command palette; the per-profile layout under the user data dir was not printed on fetched pages).
- The full current settings key list for agent mode (VS Code settings reference is version-moving; only the keys above were pinned to sources).
- Whether agent mode honors `COPILOT_HOME` for anything beyond `mcp-config.json` (only that one file is documented as read).

## Sources

- [1] https://code.visualstudio.com/docs/agent-customization/mcp-servers: mcp.json locations, formats, commands, transports, devcontainer block
- [2] https://code.visualstudio.com/docs/agent-customization/custom-instructions: instruction files, applyTo, useInstructionFiles, ~/.copilot/copilot-instructions.md
- [3] https://code.visualstudio.com/docs/configure/command-line: --user-data-dir, --profile, version
- [4] https://code.visualstudio.com/docs/configure/settings: settings scopes and profiles
- [5] https://code.visualstudio.com/updates/v1_99: agent mode GA in Stable, `chat.agent.enabled`
- [6] https://code.visualstudio.com/docs/agents/reference/ai-settings: `chat.agent.enabled` in the AI settings reference
- [7] https://code.visualstudio.com/docs/enterprise/manage-ai-settings: `ChatAgentMode` enterprise policy
