# Subplan 01 — loss-minimizing document engine

Parent: [master plan](master-plan.md)  
Task prefix: DOC  

Status: complete. DOC-01/03..07 were verified at baseline by the group-A verifier and judge;
area 1 closed DOC-02, DOC-08, DOC-09, and DOC-10 with judge verification.

What shipped: the strict JSON, JSONC, TOML, YAML, env, and managed-span codecs in superai-config,
each with an explicit read, write, or read-only decision, plus the typed edit executor, schema
validation hooks, and diff model with formatting-change warnings.

## Exit gate

- [x] All six required format classes have explicit read/write/read-only support decisions.
- [x] No-op byte identity passes.
- [x] Unknown data and supported lexical material survive edits.
- [x] Unsupported ambiguous constructs fail before mutation planning.
- [x] Adapters can express every owned edit without format-specific logic in core workflows.

## Residuals

> Completion record (run 3): DOC-01/03..07 verified at baseline (group-A verifier/judge);
> DOC-02 executor, DOC-08 managed spans, DOC-09 schema/deprecation hooks, and DOC-10
> formatting-change warnings closed by area 1 (judge-verified: executor policy enforcement,
> span fail-closed codec, validate_with_schema, DiffResult.formatting_warnings). JSONC/YAML
> refuse changing writes with typed LossyWrite — the codecs' explicit read-only decision per
> the DOC-05/06 preservation-policy fallback, not a gap. TOML executor refuses array-of-tables
> identity edits with a typed UnsupportedOperation (no production caller needs them).
