# Reasonix — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/esengine/DeepSeek-Reasonix](https://github.com/esengine/DeepSeek-Reasonix) (MIT), the main-branch README (legacy TypeScript line) and npm[1][2]. Reasonix is a DeepSeek-native terminal coding agent engineered around DeepSeek's prefix-cache: a cited real-world day ran 435M input tokens at 99.82% cache hit. Two lines coexist: the TypeScript 0.x line on `main` is in maintenance mode (bug fixes only), and active development is the Go rewrite on `main-v2`, the new default branch; the nix index builds the Go line (v2.35.0 pin, current tag studio-v2.37.0 of 2026-10-12)[1][3]. The desktop client is a Tauri app sharing the same config. Pushed 2026-10-12: alive.

---

## 1. Install, executable, version

```bash
npm install -g reasonix        # also drops a `dsnix` shim; `npm i -g dsnix` does the same
reasonix code my-project       # paste a DeepSeek API key on first run; it persists
npx reasonix code              # latest package without installing
```

Subcommands: `code [dir]` (coding agent, default), `chat` (tools-off), `run "task"` (one-shot streaming to stdout, for pipes), `doctor` (Node, API key, MCP wiring), `update`, plus `replay`, `diff`, `events`, `stats`, `index`, `mcp`, `prune-sessions`[1]. DeepSeek keys come from platform.deepseek.com; no other provider is supported by design[1].

## 2. Config files

One JSON file at `~/.reasonix/config.json`, with per-project overrides under `<project>/.reasonix/`[1]. The full key reference lives in the hosted configuration guide (bilingual); the README's own summary of what it covers[1]:

| Topic | Behavior |
|---|---|
| MCP servers | stdio, SSE, and Streamable HTTP; one spec format for `config.json` and `--mcp` |
| Skills | Markdown playbooks, `inline` or `subagent` mode |
| Memory | User-private knowledge pinned into the prefix; `user` / `feedback` / `project` / `reference` types |
| Hooks | Shell commands on `PreToolUse` (gating), `PostToolUse`, `UserPromptSubmit`, `Stop` |
| Permissions | Per-workspace shell allowlist, exact-prefix match |
| Web search | Bing default; switchable to Baidu AI Search, SearXNG, Metaso, Tavily, Perplexity, Exa, Brave, or Ollama via `/search-engine` |
| Semantic index | `reasonix index` on local Ollama or any OpenAI-compatible embedding endpoint |

## 3. Skills

```bash
/skill new my-skill              # <project>/.reasonix/skills/my-skill.md
/skill new my-skill --global     # ~/.reasonix/skills
/skill list
```

`runAs: subagent` in a skill's frontmatter spawns an isolated subagent loop instead of inlining. Claude-format skills also load: `<project>/.claude/skills/<name>/SKILL.md` and `~/.claude/skills/` are read alongside native paths, which overlaps the claude-code harness's skill tree[1].

## 4. Sessions and workspaces

Filesystem tools scope to the launch directory; `--dir` retargets, and mid-session switching is unsupported by design (memory paths would tangle with stale roots)[1]. Sessions persist per directory; the desktop client (prerelease, installers on GitHub Releases) shares the same `~/.reasonix` config and bundles its own Node runtime[1].

## 5. Not verified upstream

- The config.json key schema (the reference page was linked, not fetched; the README table above is its own summary).
- A config-dir relocation env var (none documented; the root is fixed at `~/.reasonix`).
- Differences between the TS 0.x and Go v2 config surfaces beyond what the migration guide covers (not fetched).
- Exit codes for `reasonix run`.

## Sources

- [1] https://raw.githubusercontent.com/esengine/DeepSeek-Reasonix/main/README.md: maintenance-mode note and main-v2 pointer, install, subcommands, config layout, skills and .claude compat, workspace scoping, desktop
- [2] https://github.com/esengine/DeepSeek-Reasonix/tree/main-v2: Go rewrite branch (new default)
- [3] https://github.com/esengine/DeepSeek-Reasonix/releases: latest tag studio-v2.37.0 (2026-10-12); nix pin v2.35.0
- [4] https://github.com/Qumulo/llm-agents/tree/main/packages/reasonix: nix index entry (Go build flags, owner esengine)
