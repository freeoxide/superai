# Oh My OpenCode (OpenCode plugin) — Configuration Reference

**Compiled:** 2026-10-12 against the opensoft/oh-my-opencode README (fetched via raw.githubusercontent.com), the npm registry, and the GitHub API. Oh My OpenCode is a batteries-included plugin for OpenCode (sst/opencode): async subagents, a roster of curated agents, LSP/AST tools, curated MCP servers, and a Claude Code compatibility layer for hooks, commands, skills, and agents[1]. The naming situation is the headline fact for this entry: the original repo `code-yeongyu/oh-my-opencode` was renamed to `code-yeongyu/oh-my-openagent` and pivoted into OmO, a standalone CLI on a fork of pi; the npm name `oh-my-opencode` (currently 5.1.30) now publishes that CLI, not the plugin[2][3]. The plugin lineage survives as the fork opensoft/oh-my-opencode, whose README still describes plugin v3.0 as stable[1][3]. The standalone OmO CLI is a separate catalog candidate (`omo-ai`); this file covers the OpenCode-plugin product.

---

## 1. Install, executable, version

The plugin has no executable of its own; it loads inside OpenCode. Install path from the v3.0-era README: install the `oh-my-opencode` npm package (`oh-my-opencode@latest` at that time), with the recommended flow being to paste an install prompt into any coding agent pointing at the upstream installation guide[1]:

```
curl -s https://raw.githubusercontent.com/code-yeongyu/oh-my-opencode/refs/heads/master/docs/guide/installation.md
```

Version probe: `opencode --version` (the README notes OpenCode must be newer than 1.0.132 to avoid a config bug)[1]. Uninstall removes the plugin entry from the OpenCode config and deletes the two `oh-my-opencode.json` files below[1].

## 2. Config files

| Path | Purpose | Source |
|---|---|---|
| `~/.config/opencode/opencode.json[c]` | OpenCode config; the `plugin` array gains `"oh-my-opencode"` | [1] |
| `.opencode/oh-my-opencode.json` | Plugin config, project scope | [1] |
| `~/.config/opencode/oh-my-opencode.json` | Plugin config, user scope | [1] |

Format is JSONC (comments and trailing commas allowed). Documented plugin-config keys: agent overrides (models, temperatures, prompts, permissions), background task concurrency per provider and model, task categories (`visual`, `business-logic`, custom), and `disabled_hooks`[1].

## 3. Agents, skills, MCP, hooks

- Agents shipped by the plugin: Sisyphus (main, Opus 4.5 High), Prometheus (planner), Metis (plan consultant), Oracle (design/debugging, GPT 5.2 Medium), Frontend UI/UX Engineer (Gemini 3 Pro), Librarian (Sonnet 4.5), Explore (Grok Code), Multimodal Looker[1].
- Skills: `playwright` (browser automation) and `git-master` (atomic commits)[1].
- MCP: curated servers Exa (web search), Context7 (docs), and Grep.app (code search), plus Claude Code MCP compatibility[1]. Configuration lives inside OpenCode's own `mcp` surface (see [opencode.md](opencode.md)).
- Hooks: 25+ built-in hooks across PreToolUse, PostToolUse, UserPromptSubmit, and Stop, plus a Todo Continuation Enforcer, Comment Checker, Ralph Loop, and tmux integration[1].
- Trigger: prompts containing `ultrawork` (or `ulw`) engage the full orchestration[1].

## 4. Multi-instance wrappers

The plugin owns no state outside OpenCode's config tree, so isolation reduces to OpenCode's own levers (`OPENCODE_CONFIG` / `XDG_CONFIG_HOME`, per [opencode.md](opencode.md)). One caveat is honest to record: the README's env-vars section was listed but not rendered on the fetched page, so any plugin-specific env knobs are unverified.

## 5. Not verified upstream

- The contents of the README's Environment Variables section (TOC entry only in fetched text).
- Whether the opensoft fork (last push 2026-01-26, license file present but not a standard SPDX id) is maintained or merely archived; upstream activity moved to oh-my-openagent.
- Current install instructions for the plugin specifically, since the npm name now serves the OmO CLI; the v3.0-era guide URLs are the last known-good source.
- The claim on third-party blogs of an `oh-my-opencode-slim` variant; not found on the fetched upstream pages.

## Sources

- [1] https://raw.githubusercontent.com/opensoft/oh-my-opencode/master/README.md: plugin description, config paths, agents, skills, MCPs, hooks, install/uninstall
- [2] https://registry.npmjs.org/oh-my-opencode: package metadata, repository now pointing at code-yeongyu/oh-my-openagent
- [3] https://github.com/opensoft/oh-my-opencode: fork status (parent code-yeongyu/oh-my-openagent), 1.2k stars, default branch dev
- [4] https://github.com/code-yeongyu/oh-my-openagent: renamed upstream, now the OmO standalone CLI (separate catalog entry)
