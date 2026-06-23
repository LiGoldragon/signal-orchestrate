# INTENT — signal-orchestrate

*The ordinary wire vocabulary contract for Persona orchestration. Defines the
typed request/reply/event channel that the `orchestrate` CLI and peers use to
claim, release, hand off, and observe roles, and to submit and query the activity
log. Companion to `ARCHITECTURE.md` and `Cargo.toml`. Maintenance: `primary/skills/repo-intent.md`.*

## Repo-scope only

This file carries only the intent that is FOR this `signal-orchestrate` contract.
Workspace-shape intent stays in the primary workspace `primary/INTENT.md`.
Component daemon intent stays in `orchestrate/INTENT.md`. Meta orchestration
policy stays in `meta-signal-orchestrate`.

## Why this repo exists

`signal-orchestrate` is the **ordinary peer-callable wire contract** for the
`orchestrate` daemon. It carries the role-lifecycle relation (claim, release,
handoff, observation), the activity-log relation (submit, query), and the
workflow-execution relation used by criome guard contracts. It contains no
daemon, actor, database, CLI parser, or transport policy — the daemon lowers
these public operations to component-local commands and effect observations
internally.
Meta orchestration policy (lane registry, scheduling, supervision policy)
stays in `meta-signal-orchestrate`; runtime actors, the store, and the lowering
logic live in `orchestrate`.

## The channel shape

The Orchestrate channel carries:

- **Role lifecycle:** `Claim(RoleClaim)`, `Release(RoleRelease)`,
  `Handoff(RoleHandoff)`, `Observe(Observation)`.
- **Activity log:** `Submit(ActivitySubmission)`, `Query(ActivityQuery)`.
- **Workflow execution:** `RunWorkflow(WorkflowRunRequest)` names a
  content-addressed workflow, criome contract, and authorized object reference;
  `ObserveWorkflowRun` / `WorkflowRunObservationRetraction` subscribe to and
  close the run stream.
- **Observation subscription:** `Watch(ObservationSubscription)` opening an
  `ObservationStream`, `Unwatch(ObservationToken)` closing it.
- **Replies:** `ClaimAcceptance` / `ClaimRejection` (carrying typed
  `ScopeConflict` records), `ReleaseAcknowledgment`, `HandoffAcceptance` /
  `HandoffRejection`, `RoleSnapshot`, `LanesObserved`, `ActivityAcknowledgment`,
  `ActivityList`, workflow acceptance / receipt / log replies,
  `PartialApplied`, and the observation open/close replies.
- **Events:** `WorkflowRunUpdated(WorkflowRunUpdate)` on the
  `WorkflowRunStream`; `Observed(ObservationEvent)` on the
  `ObservationStream`.

The workflow surface is the local execution-chamber half of criome guard
contracts. Orchestrate runs or coordinates agent/model steps and returns
content-addressed logs and `signal-criome::WorkflowReceipt` values. Criome owns
the guard contract and authorization verdict adoption; this crate only names
the public wire shape for starting, observing, and reporting a workflow run.

The wire vocabulary is contract-local: the daemon lowers these public operations
into component-local Nexus commands and SEMA reads or writes. Database-action
classification never crosses this public wire.

## Channels are closed, boundaries are named

- Wire enums are closed. No `Unknown` escape hatch; partial application is a
  typed `PartialApplied` reply, not an open escape.
- Request payloads do not mint activity timestamps, claim sequence, or daemon
  identity — the daemon store supplies them when committing.
- `orchestrate` mints those values at the daemon; request records carry the
  claimed scopes, the role, and the submitted activity body only.
- No stringly-typed dispatch. Scope references, role names, and rejection
  reasons are typed closed enums.

## Wire vocabulary discipline

Per `primary/skills/contract-repo.md` §"Public contracts use contract-local
operation verbs":

- Operation roots are domain verbs in verb form: `Claim`, `Release`, `Handoff`,
  `Observe`, `Submit`, `Query`, `Watch`, `Unwatch` — not Sema class words.
- Reply success variants name the outcome: `Claim` → `ClaimAcceptance`;
  rejections carry typed records (`ClaimRejection` with `ScopeConflict`,
  `HandoffRejection`).
- Payload record names are domain nouns the operation carries (`RoleClaim`,
  `RoleRelease`, `ActivitySubmission`, `ActivityQuery`), not `Request` or
  generic containers.
- The mandatory `Tap`/`Untap` observable surface (macro-injected) is the
  standardized observability hook for this persona component; the domain
  `Watch`/`Unwatch` pair carries the role-observation subscription alongside it.

## Constraints

- This crate carries only typed wire vocabulary, NOTA codecs, and round-trip
  witnesses.
- No runtime code: no actors, no tokio, no socket binding, no redb, no claim or
  scheduling logic.
- Contract types derive NOTA in this crate. Consumers do not carry shadow types.
- Every operation, reply, and event variant round-trips through both rkyv frames
  and NOTA text.
- Activity timestamps and claim sequence are not accepted from callers; the
  daemon store supplies them at commit.
- Meta orchestration authority (lane registry, scheduling policy,
  supervision policy) stays in `meta-signal-orchestrate`, not here.

## Daemon lowering boundary

The contract names the public action at the boundary. The daemon decides what
internal work, durable read, durable write, effect, rejection, or reply each
action becomes. Public contracts do not mirror `Assert`, `Mutate`, `Retract`,
`Match`, `Subscribe`, or `Validate`, and this crate does not depend on
`signal-sema`.

## Code map

```text
src/lib.rs                           — RoleClaim/RoleRelease/ActivitySubmission records, NOTA codecs, signal_channel! invocation
schema/signal-orchestrate.concept.schema — concept-schema source for the contract
tests/round_trip.rs                  — rkyv frame and NOTA round-trip witnesses per operation
```

## Non-ownership

This crate does not own:

- `orchestrate` daemon runtime, Kameo actors, or component lifecycle;
- the orchestrate redb store, claim tables, or activity log storage;
- socket binding, transport, or version handshake policy;
- claim conflict resolution, scheduling, lane registry, or supervision logic;
- meta orchestration policy (that is `meta-signal-orchestrate`);
- CLI formatting, audit wrapping, or Nexus record composition.

## See also

- `ARCHITECTURE.md` — detailed channel shape, the contract-local-verb migration,
  the mandatory observable surface, and closed-enum discipline.
- `../orchestrate/INTENT.md` — daemon-side intent (schema-driven planes, actors,
  state, policy lowering).
- `../meta-signal-orchestrate/INTENT.md` — meta orchestration policy contract.
- `primary/skills/contract-repo.md` — contract repo discipline and naming rules.
- `primary/skills/component-triad.md` — repo triad structure and wire layers.
