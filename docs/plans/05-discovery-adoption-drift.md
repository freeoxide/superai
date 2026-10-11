# Subplan 05 — discovery, adoption, ownership, and drift

Parent: [master plan](master-plan.md)  
Task prefix: DRF  

DRF-01..08 are closed. The run 3 judge verified the work, plus a final gate-fix round.

Shipped in crates/superai-core: bounded adapter-driven discovery, config-root
fingerprinting, wrapper digest checks, foreign and orchestrator ownership detection,
record-first adoption, orphan quarantine, and a 15-category risk-ranked drift report.

## Exit gate

- [x] Candidate discovery is bounded and adapter-driven.
- [x] Foreign ownership has evidence and blocks mutation.
- [x] Adoption is record-first and config-preserving.
- [x] Orphan handling is explicit/recoverable.
- [x] Drift report covers config, binary, wrapper, version, fixed-path, and daemon state.

## Completion record

> Completion record (run 3): DRF-01..08 closed by area 6b (judge-verified: ScanOptions user
> roots/wrapper dirs with deadline + surfaced permission diagnostics + glob expansion;
> version-marker/binary-adjacency signals; wrapper-dir scan; mise/asdf shim detection with
> typed AmbiguousOwnership blocking adopt/register/quarantine; marker-first reconciliation;
> OrphanWrapperChoice + quarantine_unmanaged_root; 15-category drift report with risk and
> next-operations — `drift_report_groups_by_harness_with_risk_and_next_ops`). DRF-06 adoption
> was already implemented at baseline (12 tests).
>
> Final-gate-fix round: DRF-04's orchestrator-managed detector landed
> (`detect_orchestrator_manager`: the documented Vibe Kanban
> `.vibe-kanban-workspaces/`, Conductor `~/conductor/workspaces/` (+ a bounded
> `~/.conductor/settings.toml` reference check), and Sculptor
> `~/.sculptor/workspaces/` markers classify the candidate ForeignManaged with the
> orchestrator named and block adoption — test
> `orchestrator_workspaces_are_foreign_managed_and_block_adoption`), and the drift
> classifier's lenient digest-substring fallback was removed: wrapper health now requires
> parseable-marker + digest equality (`is_owned_wrapper`), the same strict standard the
> repair path compares against, so an edited wrapper that merely still contains the digest
> string reports WrapperChanged instead of healthy.
