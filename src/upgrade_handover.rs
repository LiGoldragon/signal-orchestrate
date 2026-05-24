//! No-downtime upgrade — handover protocol skeleton for `orchestrate`.
//!
//! Phase 4 deliverable of the orchestrate port (second-designer/173).
//! Designs and sketches — but does not implement the full daemon
//! state machine — the no-downtime upgrade story for the orchestrate
//! component. Implementation lands when the macro emits the typed
//! `VersionProjection` impls and the runtime crate has both
//! current-version + next-version daemons compiled side-by-side.
//!
//! ## Why orchestrate's no-downtime story matters
//!
//! `orchestrate` is the workspace's **lane-claim authority**. Brief
//! downtime means agents cannot claim lanes — every parallel agent
//! is blocked until the daemon comes back up. Spirit's downtime
//! reaches "writes briefly buffered, no permanent loss"; orchestrate's
//! downtime reaches "every running agent can't acquire scope until
//! the daemon is back." The bar is higher.
//!
//! ## Protocol shape — the three candidate designs
//!
//! Three protocols were evaluated. The handover protocol settled on
//! below is **drain-with-mirror** — variant 3 below — because it
//! preserves both lane-claim authority continuity AND avoids the
//! socket-binding race of atomic transfer.
//!
//! ### Variant 1 — Mirror mode (rejected)
//!
//! New daemon shadows old daemon's state via the old daemon's read
//! API. Both daemons serve writes; a per-write conflict-resolution
//! pass reconciles divergence. Rejected because two writers can
//! grant the SAME claim to two roles, and the conflict-resolution
//! pass cannot retract a claim already acted upon (an agent reads
//! the lock, edits files, then learns its claim was revoked).
//! The two-writer window is the failure mode the lane-claim
//! authority must not allow.
//!
//! ### Variant 2 — Atomic socket transfer (rejected)
//!
//! New daemon binds new socket; old daemon unbinds; the systemd unit
//! flips the symlink. Rejected because clients holding open
//! connections to the old socket continue to use it — they don't
//! discover the new socket until their next connect, which can be
//! hours later. The "instant flip" is in fact a slow rollover, and
//! during the rollover SOME clients see old state and SOME see new.
//!
//! ### Variant 3 — Drain-with-mirror (chosen)
//!
//! The next-version daemon starts in cold-standby mode. The handover
//! orchestrator (Persona — pending) tells the current daemon to
//! enter drain mode: stop accepting NEW claims; keep serving reads;
//! keep accepting release/handoff on existing claims; mirror every
//! accepted write to the next-version daemon via the private upgrade
//! socket. When the next-version daemon has caught up to a commit
//! marker the current daemon agrees with, Persona flips the active-
//! version selector. New connections land on next. The current
//! daemon keeps serving its OWN open connections until they close,
//! mirroring every state change to next via the upgrade socket
//! during this graceful-drain window. When the last connection
//! closes, the current daemon retires.
//!
//! Properties:
//! - **No two-writer window** — drain prevents the current daemon
//!   from accepting new claims; next daemon doesn't accept ANY
//!   writes until selector flip.
//! - **No connection breakage** — existing clients keep their
//!   connections to current; new clients land on next.
//! - **No state divergence** — current mirrors every accepted
//!   state change to next via the upgrade socket; next applies them
//!   AFTER selector flip during the drain window so its own state
//!   stays consistent.
//! - **Bounded handover time** — drain mode is at most as long as
//!   the longest open connection's lifetime, which is bounded by
//!   the workspace's session-end discipline.
//!
//! ## State machine for the current-version daemon
//!
//! ```text
//!     Active
//!       │
//!       │  owner: BeginDrain { target_version }
//!       ▼
//!     Draining       ← stop accepting new Claims
//!       │            ← keep serving Reads
//!       │            ← keep accepting Release/Handoff for existing claims
//!       │            ← mirror every accepted Mutate to next via upgrade socket
//!       │
//!       │  next: ReadyToHandover { source_marker N }
//!       │  current: HandoverAccepted
//!       │  selector: flipped to next
//!       ▼
//!     PostFlip       ← new connections land on next
//!       │            ← existing connections keep using current
//!       │            ← every accepted state change mirrors to next
//!       │
//!       │  last existing connection closes
//!       ▼
//!     Retired        ← all sockets closed; process exits
//! ```
//!
//! ## State machine for the next-version daemon
//!
//! ```text
//!     Standby
//!       │
//!       │  current: BeginHandover request on upgrade socket
//!       ▼
//!     Hydrating      ← read current's state at commit marker N
//!                    ← project every record through VersionProjection
//!                    ← refuse all ordinary + owner socket traffic
//!       │
//!       │  hydration complete at marker N
//!       │  current: WritesPauseAck (stops accepting new Mutates)
//!       │  next: ReadyToHandover { source_marker N }
//!       │  current: HandoverAccepted
//!       │  selector: flipped
//!       ▼
//!     Catching       ← accept mirrored writes from current via upgrade socket
//!                    ← serve new ordinary + owner socket traffic
//!       │
//!       │  current: AllConnectionsClosed
//!       ▼
//!     Active         ← stand-alone; current daemon retired
//! ```
//!
//! ## Wire-level operations on the upgrade socket
//!
//! The upgrade socket is the private daemon-to-daemon channel
//! introduced by `signal-version-handover` (per
//! `/home/li/primary/reports/designer/287-version-handover-component-explained.md`).
//! Operations specific to orchestrate's drain protocol:
//!
//! | Operation | Direction | Purpose |
//! |---|---|---|
//! | `AskHandoverMarker` | next → current | request current's commit marker N |
//! | `HandoverMarker` | current → next | reply with marker N + table snapshot reference |
//! | `BeginDrain` | next → current | request current enters Draining state |
//! | `DrainAck` | current → next | acknowledge drain mode entered |
//! | `MirrorWrite` | current → next | replicate one accepted Mutate from current to next |
//! | `MirrorAck` | next → current | acknowledge mirrored write applied |
//! | `ReadyToHandover` | next → current | next has caught up to marker N; ready for selector flip |
//! | `HandoverAccepted` | current → next | current acknowledges; safe to flip selector |
//! | `AllConnectionsClosed` | current → next | current's last connection closed; current retiring |
//!
//! ## Open questions for the operator
//!
//! - **Selector-flip mechanism.** Is the selector a symlink, a
//!   systemd unit selector, or a process-local registry? Spirit's
//!   v0.1.0 → v0.1.1 cutover uses the CriomOS-home symlink today;
//!   `reports/designer/287` §5 marks this as pending — the
//!   active-version selector belongs to Persona (bead
//!   `primary-a5hu`) once Persona exists.
//! - **What about a fresh CLI invocation during drain?** A new
//!   `orchestrate '(Claim ...)'` lands on the next daemon's
//!   ordinary socket via the selector. Since next is in `Catching`
//!   state during drain, it can serve the new claim — claims
//!   against records mirrored from current are already typed
//!   correctly via VersionProjection. The only window where new
//!   claims are refused is the brief `WritesPauseAck` → selector
//!   flip period — typically a few seconds.
//! - **What about lock-file projection?** Lock files are
//!   projections of typed state per the orchestrate ARCHITECTURE
//!   §6. During drain both daemons project to the SAME lock-file
//!   directory; the next daemon's projection is the truth after
//!   selector flip. Race condition: both daemons write the same
//!   path. The fix is for current to STOP projecting once it
//!   sends `HandoverAccepted` — next becomes the sole projector
//!   from that moment.

use std::time::Duration;

// ─── State machines ───────────────────────────────────────

/// State of the **current-version** orchestrate daemon during
/// a no-downtime upgrade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CurrentDaemonState {
    Active,
    Draining,
    PostFlip,
    Retired,
}

impl CurrentDaemonState {
    pub fn transition(self, event: CurrentDaemonEvent) -> Result<Self, HandoverError> {
        match (self, event) {
            (Self::Active, CurrentDaemonEvent::BeginDrain) => Ok(Self::Draining),
            (Self::Draining, CurrentDaemonEvent::HandoverAccepted) => Ok(Self::PostFlip),
            (Self::Draining, CurrentDaemonEvent::HandoverAborted) => Ok(Self::Active),
            (Self::PostFlip, CurrentDaemonEvent::LastConnectionClosed) => Ok(Self::Retired),
            (state, event) => Err(HandoverError::InvalidTransition { from: state, event }),
        }
    }

    pub fn accepts_new_claims(self) -> bool {
        matches!(self, Self::Active)
    }

    pub fn accepts_existing_claim_mutations(self) -> bool {
        matches!(self, Self::Active | Self::Draining | Self::PostFlip)
    }

    pub fn projects_lock_files(self) -> bool {
        matches!(self, Self::Active | Self::Draining)
    }
}

/// State of the **next-version** orchestrate daemon during a
/// no-downtime upgrade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NextDaemonState {
    Standby,
    Hydrating,
    Catching,
    Active,
}

impl NextDaemonState {
    pub fn transition(self, event: NextDaemonEvent) -> Result<Self, HandoverError> {
        match (self, event) {
            (Self::Standby, NextDaemonEvent::HandoverBegun) => Ok(Self::Hydrating),
            (Self::Hydrating, NextDaemonEvent::HydrationComplete) => Ok(Self::Catching),
            (Self::Catching, NextDaemonEvent::CurrentRetired) => Ok(Self::Active),
            (state, event) => Err(HandoverError::InvalidNextTransition { from: state, event }),
        }
    }

    pub fn accepts_ordinary_traffic(self) -> bool {
        matches!(self, Self::Catching | Self::Active)
    }

    pub fn applies_mirror_writes(self) -> bool {
        matches!(self, Self::Catching)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CurrentDaemonEvent {
    BeginDrain,
    HandoverAccepted,
    HandoverAborted,
    LastConnectionClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NextDaemonEvent {
    HandoverBegun,
    HydrationComplete,
    CurrentRetired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoverError {
    InvalidTransition {
        from: CurrentDaemonState,
        event: CurrentDaemonEvent,
    },
    InvalidNextTransition {
        from: NextDaemonState,
        event: NextDaemonEvent,
    },
}

// ─── Wire-protocol shapes ─────────────────────────────────

/// Wire-protocol operation shapes for the private upgrade socket.
/// The actual definitions live in `signal-version-handover` — these
/// are the skeleton types that orchestrate's daemon will speak.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitMarker(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoverMarkerReply {
    pub marker: CommitMarker,
    pub table_snapshot_reference: TableSnapshotReference,
}

/// Opaque reference to a snapshot of the current daemon's tables.
/// The next daemon reads through this reference to copy state.
/// Implementation: a redb savepoint identifier plus the table set
/// to read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableSnapshotReference {
    pub savepoint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirrorWritePayload {
    pub commit_marker: CommitMarker,
    pub table: crate::schema_emitted::SemaTable,
    pub record_bytes: Vec<u8>,
}

/// Per-orchestrate operation latency targets during handover.
/// These are direction, not pinned numbers — implementation
/// validates them against measured drain times.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandoverLatencyTargets {
    /// Max time current spends in `WritesPauseAck` → selector
    /// flip. Bounded by the size of the in-flight write queue.
    pub write_pause_window: Duration,
    /// Max wall-clock time for hydration. Bounded by table size.
    pub hydration_timeout: Duration,
    /// Soft target for the drain window's total length. If
    /// exceeded, operator intervenes (Persona's
    /// `ForceCloseLingeringConnections`).
    pub drain_soft_target: Duration,
}

impl HandoverLatencyTargets {
    pub const fn default_targets() -> Self {
        Self {
            write_pause_window: Duration::from_millis(500),
            hydration_timeout: Duration::from_secs(30),
            drain_soft_target: Duration::from_secs(300),
        }
    }
}

// ─── Implementation skeleton ──────────────────────────────

/// Hook the daemon implements to project a record from current-
/// version shape to next-version shape during hydration. The actual
/// `VersionProjection` trait lives in the `version-projection` crate;
/// orchestrate's per-type impls land in a sibling `migration.rs`
/// module following the Spirit pattern at
/// `signal-persona-spirit/src/migration.rs`.
pub trait OrchestrateVersionProjection {
    type CurrentClaim;
    type NextClaim;
    type CurrentActivity;
    type NextActivity;
    type CurrentRole;
    type NextRole;
    type Error;

    fn project_claim(&self, source: Self::CurrentClaim) -> Result<Self::NextClaim, Self::Error>;
    fn project_activity(
        &self,
        source: Self::CurrentActivity,
    ) -> Result<Self::NextActivity, Self::Error>;
    fn project_role(&self, source: Self::CurrentRole) -> Result<Self::NextRole, Self::Error>;
}

// ─── Smoke tests for the state machines ───────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_daemon_walks_active_to_retired() {
        let state = CurrentDaemonState::Active;
        assert!(state.accepts_new_claims());

        let state = state.transition(CurrentDaemonEvent::BeginDrain).unwrap();
        assert_eq!(state, CurrentDaemonState::Draining);
        assert!(!state.accepts_new_claims());
        assert!(state.accepts_existing_claim_mutations());
        assert!(state.projects_lock_files());

        let state = state
            .transition(CurrentDaemonEvent::HandoverAccepted)
            .unwrap();
        assert_eq!(state, CurrentDaemonState::PostFlip);
        assert!(state.accepts_existing_claim_mutations());
        assert!(!state.projects_lock_files());

        let state = state
            .transition(CurrentDaemonEvent::LastConnectionClosed)
            .unwrap();
        assert_eq!(state, CurrentDaemonState::Retired);
        assert!(!state.accepts_existing_claim_mutations());
    }

    #[test]
    fn next_daemon_walks_standby_to_active() {
        let state = NextDaemonState::Standby;
        assert!(!state.accepts_ordinary_traffic());

        let state = state.transition(NextDaemonEvent::HandoverBegun).unwrap();
        assert_eq!(state, NextDaemonState::Hydrating);
        assert!(!state.accepts_ordinary_traffic());
        assert!(!state.applies_mirror_writes());

        let state = state
            .transition(NextDaemonEvent::HydrationComplete)
            .unwrap();
        assert_eq!(state, NextDaemonState::Catching);
        assert!(state.accepts_ordinary_traffic());
        assert!(state.applies_mirror_writes());

        let state = state.transition(NextDaemonEvent::CurrentRetired).unwrap();
        assert_eq!(state, NextDaemonState::Active);
        assert!(state.accepts_ordinary_traffic());
        assert!(!state.applies_mirror_writes());
    }

    #[test]
    fn drain_can_be_aborted_returning_to_active() {
        let state = CurrentDaemonState::Active
            .transition(CurrentDaemonEvent::BeginDrain)
            .unwrap();
        let state = state
            .transition(CurrentDaemonEvent::HandoverAborted)
            .unwrap();
        assert_eq!(state, CurrentDaemonState::Active);
        assert!(state.accepts_new_claims());
    }

    #[test]
    fn invalid_transitions_are_typed_errors() {
        let result =
            CurrentDaemonState::Active.transition(CurrentDaemonEvent::LastConnectionClosed);
        assert!(matches!(
            result,
            Err(HandoverError::InvalidTransition {
                from: CurrentDaemonState::Active,
                event: CurrentDaemonEvent::LastConnectionClosed,
            })
        ));
    }

    #[test]
    fn latency_targets_have_sensible_defaults() {
        let targets = HandoverLatencyTargets::default_targets();
        assert!(targets.write_pause_window < targets.hydration_timeout);
        assert!(targets.hydration_timeout < targets.drain_soft_target);
    }
}
