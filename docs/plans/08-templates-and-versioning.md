# Subplan 08 — remote templates, versioning, diffs, and updates

Parent: [master plan](master-plan.md)  
Task prefix: TPL  

All six exit gates are closed. TPL-01..07 were verified at the baseline (group-E verifier/judge);
area 4 closed the remaining TPL-08 gap and pulled template use under the CAP-04 gate.

The engine is crates/superai-core: template.rs (schema, cross-validation, adapter validation),
template_fetch.rs (digest-verified fetch), template_update.rs (diff, three-way merge, apply).

## Exit gate

- [x] Direct GitHub file distribution works without registry service/package.
- [x] Historical versions are immutable and verified.
- [x] Update checks never mutate.
- [x] Three-way merge protects local divergence.
- [x] Updates are explicit and registry version advances last.
- [x] Template data cannot escape adapter selectors or execute code.

## Residuals

> Completion record (run 3): TPL-01..07 verified implemented at baseline (group-E
> verifier/judge); the remaining TPL-08 gap (deprecated-with-replacement pointer,
> major-version selector-reset policy) closed by area 4 — `replacement` field with status
> cross-validation, `UpdateStatus::Deprecated{replacement}`, deprecated-preview warnings
> naming the replacement, and major bumps demoting dropped selectors to typed
> ConflictKind::SelectorReset instead of silent auto-remove. The CAP-04 completeness gate
> now also blocks the template USE path (apply_update calls validate_against_adapter
> before any disk mutation, area-4 round 2).
