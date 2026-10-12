# Replit Agent (cloud) — Configuration Reference

**Compiled:** 2026-10-12 against docs.replit.com (agent overview, agent skills, custom instructions, MCP pages). Replit Agent builds apps from plain-language prompts inside the Replit workspace: it sets up the project, creates the app, checks its own work, and rolls back through checkpoints[1]. Everything runs on Replit's cloud; there is no local executable, no local config file, and no documented environment variable surface. This file records what configurable surfaces exist at all, which are workspace settings and project files on Replit's machines. The cloud-only shape follows the Copilot Coding Agent precedent in [copilot-cli.md](copilot-cli.md) §5: org/workspace-owned settings, nothing local to mutate.

---

## 1. Access and modes

Agent lives in the Replit Project Editor chat. Plan gating: Core and Pro plans both get Agent with all modes (Free, Auto, Power, Max); Enterprise workspaces run Auto mode only with admin-managed model controls[1]. Agent modes differ in cost and model selection; Free mode allowances reset every 5 hours, paid modes confirm each action with an "Allow once" prompt[1]. Plan mode (a checkbox in the chat input) produces an ordered task list for review before any code changes[1].

## 2. Instructions, memories, skills

| Surface | Where it lives | Source |
|---|---|---|
| Custom instructions | Workspace Settings → Customization; written once per workspace, applied across its projects and sessions | [2] |
| Memories | Retained context from past work, managed in-app | [2] |
| Installed skills | `/.agents/skills/` directory in the project, persistent across chat sessions; SKILL.md format per agentskills.io | [3] |
| Chat-attached skills | Attached per message, no install, available in every project | [3] |

Skills are created three ways: asked-for in conversation (Agent writes the skill from what it just learned), added manually (enable "Show Hidden Files" in the file sidebar, drop a markdown file into `/.agents/skills/`), or installed from a terminal via the Vercel skills CLI: `npx skills <skill> -a replit`[3]. Installed skills are project-scoped[3].

## 3. MCP and integrations

Configured at replit.com/integrations, not in files you can edit locally[4]:

- Pre-listed servers (Stripe, Linear, Notion, Sentry, Atlassian, Miro, PostHog) connect through a browser OAuth sign-in.
- Custom servers: "+ Add MCP server", a display name, and an HTTPS endpoint URL, with optional custom headers in advanced settings; "Test & save" validates the connection and walks through OAuth.
- The docs' link generator encodes `{"displayName": ..., "baseUrl": ...}` as base64 into `https://replit.com/integrations?mcp=<encoded>`.

Only HTTPS endpoints are documented; stdio is not offered. Connected tools become available across all your projects, so the scope is the account, not the project[4]. Replit also ships its own MCP server so external clients can create, find, inspect, or update Replit Apps[5]. Native connectors (BigQuery, Linear, Slack, Notion and others) are a separate mechanism from MCP[1][6].

## 4. Wrapper relevance

There is no local surface to wrap: no executable, no config directory, no env vars, no relocation knob. Isolation between identities means separate Replit accounts, and every setting above is edited through the web UI or by asking Agent itself. A superai adapter's realistic surface is: none, or at most reading `/.agents/skills/` and instruction files out of a checked-out Replit project.

## 5. Not verified upstream

- Whether the historical `replit.md` instruction file still plays a role; current docs pages do not mention it and community threads describe it as managed by the Agent itself.
- Where custom instructions and memories are stored server-side (not documented, not locally reachable regardless).
- Any API or CLI for driving Replit Agent headlessly (none found on the fetched pages).
- MCP transport details beyond HTTPS endpoints (no stdio/SSE configuration surface documented).

## Sources

- [1] https://docs.replit.com/replitai/agent: agent overview, modes, plans, plan mode, connectors
- [2] https://docs.replit.com/chat/custom-instructions: custom instructions and memories scope
- [3] https://docs.replit.com/build/use-agent-skills: skills location, creation methods, skills CLI
- [4] https://docs.replit.com/build/connect-via-mcp: MCP server configuration UI, HTTPS-only, account-wide availability
- [5] https://docs.replit.com/chat/connect-through-mcp: MCP overview, Replit's own MCP server
- [6] https://docs.replit.com/llms.txt: documentation index used to locate the pages above
