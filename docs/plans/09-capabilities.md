# Subplan 09 — capability matrix and resolution

Parent: [master plan](master-plan.md)  
Task prefix: CAP  

CAP-01..06 closed by area 4, judge-verified in rounds 1-2. Resolution reads adapter, provider, template, extension, and policy data fresh; the compile-time MATRIX is reference data only.

Shipped: InstanceId-keyed `resolve_for_instance` and `filter_instances_by_capability`,
`validate_resolution_completeness` blocking template validation and the apply path, and
capability deltas in update previews.

## Exit gate

- [x] Matrix resolves all required capability IDs for every supported pair.
- [x] Native/substituted/absent carries source and explanation.
- [x] Resolution is fresh and not stored in records.
- [x] Consumer APIs do not require harness checks.
- [x] Capability deltas appear in update previews.

## Residuals
> Completion record (run 3): CAP-01..06 closed by area 4 (judge-verified rounds 1-2,
> including both former contradictions: resolution now reads adapter declarations /
> provider data / template maps / verifiable MCP extension state / downgrade-only policy
> rows FRESH — the compile-time MATRIX is reference data only, never consulted by live
> resolution; resolve_for_instance/filter_instances_by_capability are InstanceId-keyed and
> resolve each instance against its OWN provider via explicit mapping or fresh PRV-05
> detection, honest-empty otherwise). Completeness (`validate_resolution_completeness`)
> blocks template validation AND the apply path; capability transport declarations cover
> the six ACTIVE_PAIRS harnesses — the remaining 42 adapters honestly report Unknown until
> per-harness research lands (documented breadth punt, no invented claims).
