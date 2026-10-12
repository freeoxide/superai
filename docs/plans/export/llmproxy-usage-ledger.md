# llmproxy usage-truth ledger plan

Task prefix: PRX
Estimate: L (14–18 change sets). Order within the plan matters.

## Context

llmproxy already emits exactly one usage event per request; today those events go
nowhere, because there is no persistence backend. This plan makes the proxy the
usage-truth ledger: every request persisted to SQLite with token counts, cost, and
caller attribution, queryable through an authenticated admin API. Worth having with no
consumer in mind: per-request and per-provider accounting, named client keys, retention
control, and per-provider egress routing for geo-restricted upstreams. Downstream, a
separate tool (see Companion plans) reads these endpoints to answer "how much quota is
burned and what did it cost" across machines.

Baseline, verified at commit 7d91377: six crates, per-request events already defined in
`crates/llm-proxy-storage/src/lib.rs` (`ProxyEvent`, `EventBus`, `NoopBus`), events
emitted exactly once per request in `crates/llm-proxy-server/src/routes/core_pipeline.rs:558,726,916,1033,1394,1979`,
the API crate an empty placeholder (`crates/llm-proxy-api/src/lib.rs`), no persistence
backend (the storage doc comment says SQLite "lands in a later phase"), no client
authentication, no upstream egress configuration
(`crates/llm-proxy-provider/src/transport.rs:203` builds one default reqwest client), and
CI at `.github/workflows/ci.yml` with no release pipeline.

## PRX-01 — client identity

Today the server identifies clients only by IP for rate limiting
(`crates/llm-proxy-server/src/middleware.rs:227-238`) and trusts forwarded headers only
opt-in (config `trust_forwarded_headers`). Add named client keys: `[auth.clients]` with
key ids and hashed secrets, `Authorization: Bearer <client-key>` accepted on proxy
routes, loopback-only default installs may set `auth = "none"` to preserve today's local
behavior; non-loopback binds refuse to start without client keys. `RequestReceived`
gains a `caller` field (key id, or `"anonymous"` under the loopback exemption).
Secrets are hashed at rest and never appear in events or logs; `state.rs`'s documented
"none hold secrets" invariant (comment at `crates/llm-proxy-server/src/state.rs:52`) is
updated to cover the new auth state honestly.

## PRX-02 — SQLite EventBus backend

Implement `EventBus` (`crates/llm-proxy-storage/src/lib.rs:141-144`) over SQLite
(`rusqlite` with the bundled feature, spelling and maintenance verified on crates.io per
llmproxy's AGENTS.md dependency rule). Schema: one `requests` table keyed by
`request_id` with received/completed timestamps, provider, model (requested and
upstream), route kind, client protocol, streaming flag, caller, token usage columns
(input/output/cache-read/cache-creation from the existing `Usage` type), cost (from the
existing `Cost` type), stop reason, error kind and http status for failures, latency,
and `body_hash` (already the dedup-safe digest, never the body, per the comment at
`crates/llm-proxy-storage/src/lib.rs:60-61`). Indexes on `(timestamp)`, `(provider,
timestamp)`, `(caller, timestamp)`. The writer is a dedicated task behind a bounded
channel, honoring the `emit` contract "must not block the calling task"
(`crates/llm-proxy-storage/src/lib.rs:143-144`); a full channel drops the event and
increments a counter, never backpressures a response. Retention: configurable max
age/rows, pruned on a timer. WAL mode; the database path is config under `[storage]`.

## PRX-03 — /usage admin endpoints

Fill the placeholder `llm-proxy-api` crate (`crates/llm-proxy-api/src/lib.rs`) with the
query surface, wired into the server's router next to the existing lightweight routes
(`crates/llm-proxy-server/src/routes/mod.rs:96-99`):

- `GET /usage/summary?window=…&group_by=provider|model|caller` — tokens, requests,
  cost, error counts for a time window.
- `GET /usage/requests?since=…&until=…&provider=…&limit=…` — bounded paged raw rows,
  no request bodies, ever.
- `GET /usage/keys` — caller list with last-seen (admin only).

Admin authentication and mounting: a separate `[auth.admin]` token, required for every
`/usage` route. The default mount is a second, loopback-or-tunnel-only listener
(`[server.admin_bind]`, default `127.0.0.1` on a distinct port), so the admin API is not
part of the public surface unless the operator deliberately puts the tunnel or a reverse
proxy in front of it. An explicit `admin_on_main_listener = true` opts into also serving
`/usage` on the main listener behind the same token, for setups where a second port is
not available; that opt-in is a different, named security posture and the config
validator says so. Responses are read-only; there is no write path. Prior art for the
shape: CodexBar (the macOS usage monitor whose tree `f66d18f` this plan's research
snapshotted; see its `docs/llm-proxy.md`) already ships an LLM Proxy provider that reads
an LLM-API-Key-Proxy-compatible `GET /v1/quota-stats` aggregate; serving a
quota-stats-compatible summary alongside `/usage/summary` is a cheap way to keep
CodexBar itself working against this proxy, and is noted as optional, not required.

## PRX-04 — geo-exit upstream support

Two layers, both needed when the proxy must reach geo-restricted upstreams:

1. Per-provider egress: `[provider.egress] proxy = "socks5h://host:port"` (or an http
   proxy URL) in provider TOML; the transport builds a per-provider reqwest client with
   `.proxy(...)` instead of today's single default client
   (`crates/llm-proxy-provider/src/transport.rs:203`). No proxy means today's behavior.
2. Bind-out selection stays out of scope: exit geography comes from where the tunnel
   endpoint the forward proxy listens on runs, not from llmproxy source-address tricks.

The tunnel layer and the deploy flow that configures both ends live in companion repos
(see Companion plans); this plan only honors the per-provider proxy URL.

## PRX-05 — release artifacts

There is CI (`.github/workflows/ci.yml`) but no release pipeline (no cargo-dist
config). Add cargo-dist producing musl Linux binaries plus a shell installer. Companion
deploy tooling consumes these artifacts; without them, deploy means building on the
target machine, which the plan does not want as the only path.

## PRX-06 — secret-abuse test family

llmproxy's tests have nothing named for abuse today, and this plan adds secrets, an
admin API, and a persistent ledger. Add a dedicated abuse family in llmproxy's own
suite covering: client and admin tokens never appearing in events, logs, error
responses, or the SQLite file beyond their hashed-at-rest rows; `/usage` responses
containing no request bodies; admin routes refusing unauthenticated and wrong-token
calls on both mounts; caller attribution surviving the loopback exemption without
leaking the key id of a non-anonymous caller to other callers.

## Tests

Gates: `just fmt`, `just lint`, `just test` per AGENTS.md. Event-exactly-once
assertions extended to caller attribution; SQLite writer under channel saturation drops
and counts rather than blocking; /usage endpoints refuse unauthenticated calls on both
the admin listener and the opt-in main-listener mount; retention pruning; provider
egress config validation; the PRX-06 abuse family.

## Exit gate

- [ ] llmproxy persists every request with caller attribution and serves /usage
      endpoints behind admin auth.
- [ ] Per-provider egress proxy config validated and honored by the transport.
- [ ] No client or admin token ever reaches an event, log, error response, or the
      SQLite file beyond its hashed-at-rest row; the PRX-06 family proves it.

## Out of scope

- Bind-out/source-address selection: exit geography comes from the tunnel endpoint's
  location, not from llmproxy (PRX-04 layer 2).
- Any write path on `/usage`; responses stay read-only.
- The quota-stats-compatible summary: optional, not required.

## Unverified and excluded

- Client-key rollout preferences (loopback exemption vs mandatory keys) and the admin
  token delivery mechanism on the host: the plan states its chosen defaults (loopback
  exemption in PRX-01, admin listener in PRX-03), but they are defaults this plan
  picked, not upstream decisions; they stay listed here so the discrepancy is visible
  if llmproxy's own roadmap says otherwise.

## Companion plans

The consumer-side work lives in the superai repository's subplans 15 (the Rust client
for these endpoints) and 16 (VPS deploy tooling that installs and operates the proxy).
This document's canonical home is the llmproxy repository; the copy in the superai
repo under `docs/plans/export/` is an export.
