# Freebuff — Configuration Reference

**Compiled:** 2026-10-12 against [github.com/CodebuffAI/freebuff](https://github.com/CodebuffAI/freebuff) (the README also serves as CodebuffAI/codebuff's README), freebuff.com product pages, the npm package, release assets on CodebuffAI/codebuff-community, and CLI source files in the repo[1][2][3][4]. Freebuff is Codebuff's free coding agent: five products (Desktop, CLI, Web, Cloud, Chat) with an included model catalog, no subscription, credits, or API key required; text ads fund the models[1]. The CLI is TypeScript on Bun, npm `freebuff`, binary `freebuff`. Repo pushed 2026-10-12; latest binary release tag v1.0.688 on 2026-09-08: alive.

---

## 1. Install, executable, version

```bash
npm install -g freebuff
cd ~/my-project
freebuff
```

Desktop installers come from freebuff.com/desktop; the nix index pulls `freebuff-{platform}.tar.gz` assets from CodebuffAI/codebuff-community releases[1][4]. No version-probe convention is documented on the README.

## 2. Config files

The documented-on-disk config directory comes from the CLI source, not the README[3]:

- Default root: `~/.config/manicode` (a `manicode-<env>` suffix appears on non-prod builds; the name is the project's Manicode lineage).
- Override: `FREEBUFF_CONFIG_DIR`, which must be an absolute path; the CLI throws rather than accept a relative one, so settings cannot be written relative to the current project.

## 3. Models and BYOK

The included catalog needs no key: MiMo 2.6 Flash/Pro, GLM 5.3 Flash, DeepSeek V4.1 Flash (+ Fast), GPT-6 Luna, Solar Mini/Pro 4, with paid-plan additions (Gemini 3.8 Flash, Muse Spark 1.3, GPT-6.1 Sol)[1]. BYOK is source-documented in the SDK[3]: connection entries with provider `openrouter`, `openai-compatible` (baseUrl, contextWindow, maxOutputTokens), or ChatGPT sign-in; credentials are referenced by `env:VARNAME`, `connection:` owned references, or `chatgpt:` sign-in ids, and secrets live in an OS secret service (`com.freebuff.byok.v1`) rather than config files. Freebuff Desktop can also run locally installed Claude Code and Codex agents on the user's own provider accounts[1].

## 4. Agents, ads, access

Specialized agents (file-finding, implementation, review, research, browser) split the work; Desktop isolates concurrent agents in separate workspaces[1]. Access tiers: supported regions get full access, others (and VPN users) get limited access with a daily Freebucks allowance (25/day, 20 on VPN)[1].

## 5. Not verified upstream

- The config file names inside `~/.config/manicode` (settings schema unpublished; only the directory resolution is source-verified).
- MCP support in the CLI (no MCP mention in the README; the underlying Codebuff SDK has MCP concepts, but Freebuff's surface was not verified).
- Headless/non-interactive mode for the CLI (nothing documented).
- The relationship between the `freebuff` npm package version and the codebuff-community release tags.

## Sources

- [1] https://raw.githubusercontent.com/CodebuffAI/codebuff/main/README.md: product set, model catalog, access tiers, ads, Desktop running Claude Code/Codex agents, npm install
- [2] https://github.com/CodebuffAI/freebuff: repo (pushed 2026-10-12), monorepo layout
- [3] https://raw.githubusercontent.com/CodebuffAI/freebuff/main/cli/src/utils/config-dir.ts and https://raw.githubusercontent.com/CodebuffAI/freebuff/main/sdk/src/byok.ts: FREEBUFF_CONFIG_DIR absolute-path rule, default config dir, BYOK connection schema and secret-service references
- [4] https://github.com/CodebuffAI/codebuff-community/releases: freebuff binary release assets, latest tag v1.0.688 (2026-09-08)
- [5] https://github.com/Qumulo/llm-agents/tree/main/packages/freebuff: nix index entry pinning the release channel
