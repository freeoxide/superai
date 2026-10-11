# Subplan 00 — domain foundation

Parent: [master plan](master-plan.md)  
Task prefix: FND  

All five exit gates closed at the verification-run baseline, with FND-01..07 verified
IMPLEMENTED by the group-A verifier and judge. The only residual is FND-03's optional
migration-preview API; migration itself is lossless and golden-fixture tested.

Shipped in crates/superai-core: validated identifiers, names, and path references (FND-01/02),
registry schema v1 with lossless old-record migration and golden fixtures (FND-03), the lifecycle,
ownership, isolation, and support states (FND-04), preview/result/error contracts and the error
taxonomy (FND-05/06), and the clock/ID/filesystem/platform/process/network boundaries (FND-07).

## Exit gate

- [x] Registry schema v1 documented with golden fixtures.
- [x] Current records migrate losslessly for all facts they contain.
- [x] All identifiers, names, paths, states, previews, results, and errors are validated.
- [x] No interface or harness-config value appears in instance records.
- [x] Public types have one canonical path.

## Residuals

> Completion record (run 3): all five closed at the verification-run baseline — FND-01..07
> verified IMPLEMENTED by the group-A verifier/judge (registry.rs `golden_fixtures_old_and_new_
> are_valid`, `serialization_never_emits_forbidden_fields`, `unknown_enum_and_schema_failure_
> are_actionable`, `round_trips_through_disk_keeping_foreign_keys`; ids/paths/state/operation/
> error test suites). FND-03's optional migration-preview API remains a documented minor gap;
> migration itself is lossless and golden-fixture tested.
