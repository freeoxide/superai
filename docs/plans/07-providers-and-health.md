# Subplan 07 — providers, models, API-key placement, and health

Parent: [master plan](master-plan.md)  
Task prefix: PRV  

PRV-01..08 are closed. The area 4 judge verified the work across rounds 1-2.

Shipped in crates/superai-core: the versioned provider/model schema, provider-to-harness rendering,
fresh-read effective provider inspection, provider lifecycle operations, and bounded ureq health
probes with cross-host auth stripping.

## Exit gate

- [x] Provider/model schema validated and versioned.
- [x] Standard provider addition is data-only.
- [x] API keys never enter superai records/logs.
- [x] Effective provider inspection reads fresh.
- [x] Health is bounded, redacted, protocol-aware, and opt-in.
- [x] No proxy or wire translation exists.

## Completion record

> Completion record (run 3): PRV-01..08 closed by area 4 (judge-verified rounds 1-2, incl.
> the formerly MISSING PRV-03 rendering / PRV-05 inspection / PRV-08 lifecycle and the
> schema completions: endpoint variants, headers, auth env/prefix policy, model limits/
> modalities, ProbeDefinition data, schema_version + verified_at; real bounded ureq probe
> execution with cross-host auth stripping — no live-network test in the default suite by
> design). Key-sink discipline and sentinel absence pinned by abuse/property tests;
> `data_only_adding_provider_requires_no_code_change` is the DoD-#7 ledger citation;
> `no_proxy_vault_oauth_or_chat_runtime_tokens_in_sources` guards the no-proxy invariant.
