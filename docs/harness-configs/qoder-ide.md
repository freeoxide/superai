# Qoder IDE and Desktop — Configuration Reference

**Compiled:** 2026-10-12 against docs.qoder.com (IDE settings, IDE MCP, plugin settings pages; the desktop skills and custom-agents pages) and qoder.com. Qoder is an agentic coding platform whose surfaces multiply fast: Desktop (the main app), IDE, JetBrains plugin, Mobile, CLI, Agent SDK, Cloud Agents, and QoderWake[1]. The CLI is already catalogued at [qoder-cli.md](qoder-cli.md), which owns `~/.qoder/settings.json`, `.qoder/` project files, and `QODER_CONFIG_DIR`. This file covers the IDE and Desktop surfaces only, and the headline is that they do not share the CLI's file layout: configuration is UI-driven, with one JSON editor for MCP.

Quest Mode (spec-driven work), Agent Mode with approval controls, the Repo Wiki, Projects, and Discussions are the Desktop product's working modes[1]. The docs site footer names Bright Zenith Private Limited as the copyright entity; the Alibaba lineage and the qoder.cn mainland edition are recorded in qoder-cli.md[1].

---

## 1. Install, executable, version

The Desktop app and IDE install from qoder.com/download after sign-up (email, Google, or GitHub)[1]. The JetBrains plugin installs from its marketplace. There is no documented standalone version-probe command; the app's own About/updates surface carries the version. A free two-week Pro trial per new user is followed by a Pro plan or an automatic downgrade to Free, with credit-based metering[1].

## 2. Settings surfaces

Three families, all reached through the UI[2][3]:

- Your Settings (Chat panel dropdown → Your Settings): MCP Servers, Memory (review learned habits and project context, disable automatic learning, clear memories), Rules (project coding standards), and Codebase Indexing (progress, ignored files and directories)[2].
- Plugin Settings (Qoder panel dropdown, or IDE Preferences → Qoder): Completion (enable/disable, length), NES (next-edit suggestions), Chat (tools, auto-run behavior), HTTP proxy, auto-update, and General (response language, display UI, and a configurable "local storage path", the only storage-location control mentioned anywhere)[2].
- Desktop System settings (user menu → Settings, Cmd+,): Personal entries include Models ("add and manage custom models with your own API key"), Memory, Appearance, and Voice; Integrations covers Extension Management (skills, plugins, connectors), user-level Hooks configuration, and Computer control; Coding covers Git, Worktrees, Workspace Index, SSH Connections, and Security[3].

No settings file path is documented for any family. The configurable local storage path in General is the one relocation-ish knob, and the Desktop Models entry is the BYO-model surface.

## 3. MCP

Settings → Qoder IDE Settings → MCP, with two tabs: My Servers and MCP Square (a marketplace with one-click Install)[4]. "+ Add" opens a JSON file in the `mcpServers` shape[4]:

```json
{
  "mcpServers": {
    "fetch": { "type": "sse", "url": "https://example.invalid/sse" },
    "tools": { "command": "npx", "args": ["-y", "…"], "env": { "GITHUB_TOKEN": "…" } }
  }
}
```

STDIO servers take `command`/`args`/optional `env`; SSE takes a `url`; Streamable HTTP is configured like SSE and auto-detected[4]. Saving is close-and-confirm; a link icon marks a live connection; a per-server Request Timeout dropdown exists; missing-dependency failures offer a Quick Fix[4]. Scope looks IDE-wide; no project-scope MCP file for the IDE is documented (the CLI has `.mcp.json`, this surface does not say it reads it). Enterprise admins can control MCP access[5].

## 4. Skills, memory, rules, agents (Desktop)

- Skills: Extensions → Skills → Add Skills, either "Create with Qoder" (guided) or Upload (a ZIP or a `SKILL.md` file); skills are device-local, and the docs warn that bundled scripts are executable code from the skill author[6].
- Memory: learned habits and project context, managed under Your Settings → Memory[2].
- Rules: project rules edited under Your Settings → Rules[2].
- Custom agents: Settings → Agent in the app; name, description, default model, permissions, instructions, plus attached skills, MCP servers, and plugins; per-agent tool rules (allowed tools run without approval, denied tools are blocked); Business plans (Teams and Enterprise) only[7]. Custom agent teams build on them[8]. No file format or storage path is published; creation is UI-only.

## 5. Wrapper relevance

With no documented config root, an adapter's writable surface is thin: the MCP JSON editor is the one documented file-backed write, and even its on-disk path is unpublished. Isolation levers are the General "local storage path" setting and whatever the desktop app's per-user directories turn out to be (unverified). The CLI sibling remains the practical config target for automation.

## 6. Not verified upstream

- Where the IDE/Desktop stores settings, MCP JSON, memory, rules, and custom agents on disk.
- Whether the IDE is a VS Code fork (nothing on the fetched pages says so either way; the JetBrains plugin is separate).
- Whether the IDE reads the CLI's `.mcp.json` or `~/.qoder/settings.json`.
- MCP server-entry JSON shape beyond the fetched example keys (type, url, command, args, env).
- Subagents and hooks exist as Desktop docs pages (docs.qoder.com/qoder/subagents, /qoder/hooks); their configuration mechanics were not fetched.

## Sources

- [1] https://qoder.com: product family, modes, pricing/trial, download flow, docs links
- [2] https://docs.qoder.com/plugins/settings.md: Your Settings and Plugin Settings families
- [3] https://docs.qoder.com/qoder/settings.md: desktop System settings (Personal/Integrations/Coding entries, Models with own API key, Hooks)
- [4] https://docs.qoder.com/user-guide/chat/model-context-protocol.md: IDE MCP JSON editor, transports, timeout, MCP Square
- [5] https://docs.qoder.com/account/enterprise/mcp.md: enterprise MCP access control (existence)
- [6] https://docs.qoder.com/qoder/skills.md: skills creation, SKILL.md upload, device-local scope, script warning
- [7] https://docs.qoder.com/qoder/custom-agents.md: custom agents (UI-only, Business plans, tool rules)
- [8] https://docs.qoder.com/qoder/custom-agent-teams.md: custom agent teams (existence)
