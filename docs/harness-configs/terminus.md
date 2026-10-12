# Terminus (Terminal-Bench) — Configuration Reference

**Compiled:** 2026-10-12 against tbench.ai, the Harbor framework docs, the Terminal-Bench paper coverage, and PyPI checks. Terminus is the research-preview terminal agent from the Terminal-Bench team (tbench.ai, hosted by Stanford, Harbor, and the Laude Institute), built to evaluate how well language models power autonomous agents in a terminal. Terminus 2, per the paper, "has a single tool, a headless terminal, and completes tasks using only Bash commands"[1][2][3]. This is a benchmark agent in the SWE-agent mold ([swe-agent.md](swe-agent.md)): its config surface is the evaluation harness around it, not a personal coding setup.

---

## 1. Install, executable, version

There is no user-facing install of Terminus alone. It ships as pre-integrated agents inside the Harbor framework (variants `terminus`, `terminus-1`, `terminus-2`), so installing Harbor (`uv` / pip, per the Harbor docs) is the only prerequisite; agents are "pre-integrated" and need no per-agent install[4][5]. PyPI checks on 2026-10-12 found no `terminus-agent` distribution, and the `terminus` name on PyPI is an unrelated Debian `/etc/network/interfaces` tool; do not confuse them[6]. No version-probe command is documented for Terminus itself.

## 2. Running it

Through Harbor[4][5]:

```bash
harbor run \
  --dataset terminal-bench@2.0 \
  --agent terminus \
  --model anthropic/claude-sonnet-4 \
  --ak <api-key>          # --ae KEY=VALUE injects agent environment variables
```

`--ak`/`--agent-kwarg` carries agent settings; `--ae`/`--agent-env` carries environment variables such as `OPENAI_API_KEY` or `ANTHROPIC_API_KEY` into the agent container[5]. terminus-2 supports ATIF trajectory export and MCP servers, in the sense that Harbor passes the supplied MCP configuration through to the agent[4].

## 3. Config files, env vars

No standalone config file is documented. Model credentials arrive per-run through `--ak` or `--ae` on `harbor run`[5]. Experiment configurations used in Terminal-Bench papers are published in the laude-institute/terminal-bench-experiments repository[3]. One operational caveat from the Harbor tracker: environment variables containing API keys can end up in run artifacts and logs, so wrappers must treat exported run data as secret-bearing[5].

## 4. Multi-instance wrappers

Not a personal harness; "instances" are benchmark runs, isolated by Harbor's task containers. There is no per-user state, auth store, or config tree to relocate.

## 5. Not verified upstream

- The Terminus source location (no standalone repo found; it lives inside Harbor/terminal-bench infrastructure).
- Any Terminus-specific CLI flags or env vars beyond what Harbor passes through.
- Whether `terminus-1` remains selectable in current Harbor versions (the pre-integrated agents page fetched listed terminus-2; the `terminus`/`terminus-1` variants appear in the docs' agent list and issue traffic).

## Sources

- [1] https://www.tbench.ai: Terminal-Bench home, "research-preview agent" positioning
- [2] https://arxiv.org/abs/2601.11868 (Terminal-Bench paper): Terminus 2 single-tool terminal design
- [3] https://github.com/laude-institute/terminal-bench-experiments: published experiment configurations
- [4] https://docs.harborframework.com/agents/pre-integrated-agents: terminus-2 capabilities (ATIF, MCP pass-through)
- [5] https://docs.harborframework.com and Harbor issue #1391: harbor run flags (--ak, --ae), API-key handling caveat
- [6] https://pypi.org/project/terminus/: unrelated Debian tool occupying the name (checked 2026-10-12)
