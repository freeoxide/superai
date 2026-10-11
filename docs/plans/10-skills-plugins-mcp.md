# Subplan 10 — skills, plugins, and MCP servers

Parent: [master plan](master-plan.md)  
Task prefix: EXT  

Done. Every exit-gate item passes; one EXT-10 sub-step remains unimplemented.

The skill registry acquires, updates, and removes skills without executing
content, and feeds instances through links or copies. Plugins and MCP servers
follow adapter-declared lifecycles that preserve foreign entries.

## Exit gate

- [x] Skill registry acquisition/update/remove works without executing content.
- [x] Three destination modes work where adapter supports them.
- [x] Enable/disable/remove semantics are distinct.
- [x] Copied-skill drift is conflict-aware.
- [x] Plugin lifecycle is supply-chain/ownership aware.
- [x] MCP mappings cover every writable adapter or explicit absence.
- [x] Foreign entries and secrets remain protected.

## Residual

> Completion record (run 3): EXT-01/02/03/08 verified at baseline; EXT-04/05 (config
> allow/deny + search-path mechanism, drift-checked three-way re-copy with LocallyModified
> refusal) closed by area 7; EXT-06/07/09 (adapter-declared mcp_decl/plugin_decl across
> all 48 adapters, DirectoryBundle staging with verify-then-cleanup + discovery manifests,
> four-codec MCP read path with per-entry foreign-preserving TOML writes) closed by area 5
> (judge-verified rounds 1-2: exact 33+15 MCP partition and 7+41 plugin partition
> test-enforced); EXT-10/11 (scope transfer with conflict preview, bulk cross-instance
> ops with per-target Completed/RolledBack/Refused) closed by area 7. JSONC/YAML MCP
> destinations are inspect-only with typed refusal (codec honesty). The EXT-10
> "validate via harness diagnostic command where non-mutating" sub-step is
> UNIMPLEMENTED: inspect validates by reading the destination files, which is
> sound but is not the documented diagnostic path, and the corpus DOES document
> candidate non-mutating commands for it (amp `amp mcp doctor`, auggie
> `auggie mcp list [--json]`, cursor `agent mcp list|list-tools`) — a future
> implementation should drive validation through those adapters' commands.
