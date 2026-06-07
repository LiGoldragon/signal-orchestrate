//! Signal contract — `orchestrate` CLI ↔ `orchestrate`.
//!
//! Read this file as the public interface of the workspace
//! orchestration channel. The channel carries:
//!
//! - **Role claim/release/handoff** — the claim-flow today
//!   served by `tools/orchestrate` (a bash helper); migrating
//!   into `orchestrate` per
//!   `~/primary/reports/designer/93-orchestrate-rust-rewrite-and-activity-log.md`.
//! - **Role observation** — read the active claims for every
//!   role plus the most recent activity entries.
//! - **Activity submission** — append a typed activity record:
//!   who (role), what (path or task token), why (short reason).
//!   Time is store-stamped, never agent-supplied (per
//!   `~/primary/ESSENCE.md` §"Infrastructure mints identity,
//!   time, and sender").
//! - **Activity query** — read recent activity records,
//!   optionally filtered by role or scope.
//!
//! The channel is mostly request/reply: ordinary operations
//! get typed replies, while `Watch` opens the observation
//! stream and `Unwatch` closes it. Observation events carry
//! inbound operation kinds and daemon-lowered Sema observations.
//!
//! See `ARCHITECTURE.md` for the channel's role and
//! boundaries; `~/primary/skills/contract-repo.md` for the
//! contract-repo discipline this crate follows.

use nota_codec::{
    Decoder, Encoder, NotaDecode, NotaEncode, NotaEnum, NotaRecord, NotaTransparent,
    NotaTryTransparent,
};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_frame::signal_channel;
use signal_sema::SemaObservation;
use std::fmt;
use std::str::FromStr;

pub mod schema;

// ─── Error ────────────────────────────────────────────────

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("wire path must be absolute and normalized: {path}")]
    InvalidWirePath { path: String },
    #[error("task token must be non-empty, unbracketed, and contain no whitespace: {token}")]
    InvalidTaskToken { token: String },
    #[error("scope reason must be non-empty and single-line: {reason}")]
    InvalidScopeReason { reason: String },
    #[error(
        "role identifier must be non-empty, unbracketed, and contain no whitespace or path separators: {role}"
    )]
    InvalidRoleIdentifier { role: String },
    #[error(
        "role token must be non-empty, unbracketed, and contain no whitespace or path separators: {token}"
    )]
    InvalidRoleToken { token: String },
    #[error("role vector must contain at least one token")]
    EmptyRole,
    #[error(
        "lane identifier must be non-empty, unbracketed, and contain no whitespace or path separators: {lane}"
    )]
    InvalidLaneIdentifier { lane: String },
}

// ─── Identity ─────────────────────────────────────────────

/// A dynamic workspace role identifier.
///
/// Roles are data now, not enum variants. A role is named by
/// the work context it owns, and new roles can be created at
/// runtime through the owner orchestration surface.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    NotaTryTransparent,
)]
pub struct RoleIdentifier(String);

pub type RoleName = RoleIdentifier;

impl RoleIdentifier {
    pub const CURRENT_WORKSPACE_ROLE_TOKENS: [&'static str; 11] = [
        "operator",
        "operator-assistant",
        "second-operator-assistant",
        "designer",
        "designer-assistant",
        "second-designer-assistant",
        "system-specialist",
        "system-assistant",
        "second-system-assistant",
        "poet",
        "poet-assistant",
    ];

    pub fn try_new(role: String) -> Result<Self> {
        Self::from_wire_token(role)
    }

    pub fn from_wire_token(role: impl Into<String>) -> Result<Self> {
        let role = role.into();
        if role.is_empty()
            || role.chars().any(char::is_whitespace)
            || role.contains('/')
            || role.contains('\\')
            || role.contains('[')
            || role.contains(']')
        {
            return Err(Error::InvalidRoleIdentifier { role });
        }
        Ok(Self(role))
    }

    pub fn as_wire_token(&self) -> &str {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RoleIdentifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_wire_token())
    }
}

impl FromStr for RoleIdentifier {
    type Err = Error;

    fn from_str(role: &str) -> Result<Self> {
        Self::from_wire_token(role)
    }
}

impl TryFrom<String> for RoleIdentifier {
    type Error = Error;

    fn try_from(role: String) -> Result<Self> {
        Self::from_wire_token(role)
    }
}

impl TryFrom<&str> for RoleIdentifier {
    type Error = Error;

    fn try_from(role: &str) -> Result<Self> {
        Self::from_wire_token(role)
    }
}

impl AsRef<str> for RoleIdentifier {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// One token in a role vector.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    NotaTryTransparent,
)]
pub struct RoleToken(String);

impl RoleToken {
    pub fn try_new(token: String) -> Result<Self> {
        Self::from_text(token)
    }

    pub fn from_text(token: impl Into<String>) -> Result<Self> {
        let token = token.into();
        if token.is_empty()
            || token.chars().any(char::is_whitespace)
            || token.contains('/')
            || token.contains('\\')
            || token.contains('[')
            || token.contains(']')
        {
            return Err(Error::InvalidRoleToken { token });
        }
        Ok(Self(token))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RoleToken {
    type Error = Error;

    fn try_from(token: String) -> Result<Self> {
        Self::from_text(token)
    }
}

impl TryFrom<&str> for RoleToken {
    type Error = Error;

    fn try_from(token: &str) -> Result<Self> {
        Self::from_text(token)
    }
}

impl AsRef<str> for RoleToken {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Role {
    pub tokens: Vec<RoleToken>,
}

impl Role {
    pub fn try_new(tokens: Vec<RoleToken>) -> Result<Self> {
        if tokens.is_empty() {
            return Err(Error::EmptyRole);
        }
        Ok(Self { tokens })
    }

    pub fn tokens(&self) -> &[RoleToken] {
        &self.tokens
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum LaneAuthority {
    Structural,
    Support,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    NotaTryTransparent,
)]
pub struct LaneIdentifier(String);

impl LaneIdentifier {
    pub fn try_new(lane: String) -> Result<Self> {
        Self::from_wire_token(lane)
    }

    pub fn from_wire_token(lane: impl Into<String>) -> Result<Self> {
        let lane = lane.into();
        if lane.is_empty()
            || lane.chars().any(char::is_whitespace)
            || lane.contains('/')
            || lane.contains('\\')
            || lane.contains('[')
            || lane.contains(']')
        {
            return Err(Error::InvalidLaneIdentifier { lane });
        }
        Ok(Self(lane))
    }

    pub fn as_wire_token(&self) -> &str {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LaneIdentifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_wire_token())
    }
}

impl TryFrom<String> for LaneIdentifier {
    type Error = Error;

    fn try_from(lane: String) -> Result<Self> {
        Self::from_wire_token(lane)
    }
}

impl TryFrom<&str> for LaneIdentifier {
    type Error = Error;

    fn try_from(lane: &str) -> Result<Self> {
        Self::from_wire_token(lane)
    }
}

impl AsRef<str> for LaneIdentifier {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct LaneRegistration {
    pub lane: LaneIdentifier,
    pub role: Role,
    pub authority: LaneAuthority,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum HarnessKind {
    Codex,
    Claude,
}

impl HarnessKind {
    pub const fn as_wire_token(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }

    pub fn from_wire_token(harness: impl Into<String>) -> Result<Self> {
        let harness = harness.into();
        match harness.as_str() {
            "codex" => Ok(Self::Codex),
            "claude" => Ok(Self::Claude),
            _ => Err(Error::InvalidRoleIdentifier { role: harness }),
        }
    }
}

impl fmt::Display for HarnessKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_wire_token())
    }
}

// ─── Scope reference ──────────────────────────────────────

/// What's being claimed / observed / acted on.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeReference {
    /// An absolute file or directory path.
    Path(WirePath),
    /// A bracketed task token like `[primary-f99]` (stored
    /// without brackets here).
    Task(TaskToken),
}

impl NotaEncode for ScopeReference {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::Path(path) => {
                encoder.start_record("Path")?;
                path.encode(encoder)?;
                encoder.end_record()
            }
            Self::Task(task) => {
                encoder.start_record("Task")?;
                task.encode(encoder)?;
                encoder.end_record()
            }
        }
    }
}

impl NotaDecode for ScopeReference {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        let head = decoder.peek_record_head()?;
        match head.as_str() {
            "Path" => {
                decoder.expect_record_head("Path")?;
                let path = WirePath::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::Path(path))
            }
            "Task" => {
                decoder.expect_record_head("Task")?;
                let task = TaskToken::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::Task(task))
            }
            other => Err(nota_codec::Error::UnknownVariant {
                enum_name: "ScopeReference",
                got: other.to_string(),
            }),
        }
    }
}

/// Absolute path, newtyped for cross-platform stability on
/// the wire (per `~/primary/skills/rust-discipline.md`
/// §"Newtype the wire form" — `PathBuf` archives
/// non-deterministically).
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTryTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct WirePath(String);

impl WirePath {
    pub fn try_new(path: String) -> Result<Self> {
        Self::from_absolute_path(path)
    }

    pub fn from_absolute_path(path: impl Into<String>) -> Result<Self> {
        let path = path.into();

        if !path.starts_with('/') || path.split('/').any(|component| component == "..") {
            return Err(Error::InvalidWirePath { path });
        }

        let components = path
            .split('/')
            .filter(|component| !component.is_empty() && *component != ".")
            .collect::<Vec<_>>();
        let normalized = if components.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", components.join("/"))
        };

        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for WirePath {
    type Error = Error;

    fn try_from(path: String) -> Result<Self> {
        Self::from_absolute_path(path)
    }
}

impl TryFrom<&str> for WirePath {
    type Error = Error;

    fn try_from(path: &str) -> Result<Self> {
        Self::from_absolute_path(path)
    }
}

impl AsRef<str> for WirePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A bracketed task identifier (stored without brackets).
/// Bracketed form like `[primary-f99]` is the human surface;
/// the wire carries the raw token.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTryTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct TaskToken(String);

impl TaskToken {
    pub fn try_new(token: String) -> Result<Self> {
        Self::from_wire_token(token)
    }

    pub fn from_wire_token(token: impl Into<String>) -> Result<Self> {
        let token = token.into();
        if token.is_empty()
            || token.chars().any(char::is_whitespace)
            || token.contains('[')
            || token.contains(']')
        {
            return Err(Error::InvalidTaskToken { token });
        }
        Ok(Self(token))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for TaskToken {
    type Error = Error;

    fn try_from(token: String) -> Result<Self> {
        Self::from_wire_token(token)
    }
}

impl TryFrom<&str> for TaskToken {
    type Error = Error;

    fn try_from(token: &str) -> Result<Self> {
        Self::from_wire_token(token)
    }
}

impl AsRef<str> for TaskToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ─── Reason ───────────────────────────────────────────────

/// A short reason string. Provisional per
/// `~/primary/reports/designer/92-sema-as-database-library-architecture-revamp.md`
/// §4 — strings allowed here until the typed Nexus record
/// shape for "intent" is named.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTryTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct ScopeReason(String);

impl ScopeReason {
    pub fn try_new(reason: String) -> Result<Self> {
        Self::from_text(reason)
    }

    pub fn from_text(reason: impl Into<String>) -> Result<Self> {
        let reason = reason.into();
        if reason.is_empty() || reason.contains('\n') || reason.contains('\r') {
            return Err(Error::InvalidScopeReason { reason });
        }
        Ok(Self(reason))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ScopeReason {
    type Error = Error;

    fn try_from(reason: String) -> Result<Self> {
        Self::from_text(reason)
    }
}

impl TryFrom<&str> for ScopeReason {
    type Error = Error;

    fn try_from(reason: &str) -> Result<Self> {
        Self::from_text(reason)
    }
}

impl AsRef<str> for ScopeReason {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ─── Time ─────────────────────────────────────────────────

/// Nanoseconds since the UNIX epoch. Store-supplied at
/// commit time; never agent-supplied.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    NotaTransparent,
)]
pub struct TimestampNanos(u64);

impl TimestampNanos {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

// ─── Claim verbs ──────────────────────────────────────────

/// A role asks to claim one or more scopes with a short
/// reason. Reply: `ClaimAcceptance` on success, `ClaimRejection`
/// listing every conflict on failure.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RoleClaim {
    pub role: RoleName,
    pub scopes: Vec<ScopeReference>,
    pub reason: ScopeReason,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ClaimAcceptance {
    pub role: RoleName,
    pub scopes: Vec<ScopeReference>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ClaimRejection {
    pub role: RoleName,
    pub conflicts: Vec<ScopeConflict>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ScopeConflict {
    pub scope: ScopeReference,
    pub held_by: RoleName,
    pub held_reason: ScopeReason,
}

// ─── Release verbs ────────────────────────────────────────

/// A role releases all of its currently-held scopes.
/// Reply: `ReleaseAcknowledgment` listing what was released.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RoleRelease {
    pub role: RoleName,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ReleaseAcknowledgment {
    pub role: RoleName,
    pub released_scopes: Vec<ScopeReference>,
}

// ─── Handoff verbs ────────────────────────────────────────

/// One role hands a set of scopes to another role atomically.
/// Reply: `HandoffAcceptance` on success, `HandoffRejection`
/// with a typed reason on failure.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RoleHandoff {
    pub from: RoleName,
    pub to: RoleName,
    pub scopes: Vec<ScopeReference>,
    pub reason: ScopeReason,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct HandoffAcceptance {
    pub from: RoleName,
    pub to: RoleName,
    pub scopes: Vec<ScopeReference>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct HandoffRejection {
    pub from: RoleName,
    pub to: RoleName,
    pub reason: HandoffRejectionReason,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum HandoffRejectionReason {
    /// The `from` role doesn't currently hold the named scopes.
    SourceRoleDoesNotHold,
    /// The `to` role's existing claims conflict with the
    /// scopes being handed off (the conflict list names which
    /// scopes and which existing holders).
    TargetRoleConflict(Vec<ScopeConflict>),
}

impl NotaEncode for HandoffRejectionReason {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::SourceRoleDoesNotHold => {
                encoder.start_record("SourceRoleDoesNotHold")?;
                encoder.end_record()
            }
            Self::TargetRoleConflict(conflicts) => {
                encoder.start_record("TargetRoleConflict")?;
                conflicts.encode(encoder)?;
                encoder.end_record()
            }
        }
    }
}

impl NotaDecode for HandoffRejectionReason {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        let head = decoder.peek_record_head()?;
        match head.as_str() {
            "SourceRoleDoesNotHold" => {
                decoder.expect_record_head("SourceRoleDoesNotHold")?;
                decoder.expect_record_end()?;
                Ok(Self::SourceRoleDoesNotHold)
            }
            "TargetRoleConflict" => {
                decoder.expect_record_head("TargetRoleConflict")?;
                let conflicts = Vec::<ScopeConflict>::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::TargetRoleConflict(conflicts))
            }
            other => Err(nota_codec::Error::UnknownVariant {
                enum_name: "HandoffRejectionReason",
                got: other.to_string(),
            }),
        }
    }
}

// ─── Observation ──────────────────────────────────────────

/// Select what the `Observe` operation reads.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Observation {
    Roles,
    Lanes,
}

impl NotaEncode for Observation {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::Roles => {
                encoder.start_record("Roles")?;
                encoder.end_record()
            }
            Self::Lanes => {
                encoder.start_record("Lanes")?;
                encoder.end_record()
            }
        }
    }
}

impl NotaDecode for Observation {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        let head = decoder.peek_record_head()?;
        match head.as_str() {
            "Roles" => {
                decoder.expect_record_head("Roles")?;
                decoder.expect_record_end()?;
                Ok(Self::Roles)
            }
            "Lanes" => {
                decoder.expect_record_head("Lanes")?;
                decoder.expect_record_end()?;
                Ok(Self::Lanes)
            }
            other => Err(nota_codec::Error::UnknownVariant {
                enum_name: "Observation",
                got: other.to_string(),
            }),
        }
    }
}

/// Legacy empty payload kept for older callers while the `Observe`
/// operation moves to [`Observation`].
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct RoleObservation;

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RoleSnapshot {
    pub roles: Vec<RoleStatus>,
    pub recent_activity: Vec<Activity>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct LanesObserved {
    pub lanes: Vec<LaneRegistration>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RoleStatus {
    pub role: RoleName,
    pub harness: HarnessKind,
    pub claims: Vec<ClaimEntry>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ClaimEntry {
    pub scope: ScopeReference,
    pub reason: ScopeReason,
}

// ─── Activity log ─────────────────────────────────────────

/// One activity record: who touched what and why. Time is
/// store-supplied (per ESSENCE infrastructure-mints rule —
/// the agent never invents timestamps).
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub role: RoleName,
    pub scope: ScopeReference,
    pub reason: ScopeReason,
    pub stamped_at: TimestampNanos,
}

/// Submit a new activity record. The store assigns
/// `stamped_at` on commit. Reply: `ActivityAcknowledgment`
/// carrying the slot the record landed in.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ActivitySubmission {
    pub role: RoleName,
    pub scope: ScopeReference,
    pub reason: ScopeReason,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct ActivityAcknowledgment {
    /// The slot (sequential u64) the record was assigned.
    pub slot: u64,
}

/// Query the activity log. Limit caps how many records come
/// back; filters narrow by role or scope. Empty filter list
/// = "all".
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ActivityQuery {
    pub limit: u32,
    pub filters: Vec<ActivityFilter>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum ActivityFilter {
    /// Only entries from this role.
    RoleFilter(RoleName),
    /// Only entries whose scope is `Path(p)` where `p`
    /// starts with this prefix.
    PathPrefix(WirePath),
    /// Only entries whose scope is the exact-match
    /// `Task(token)`.
    TaskToken(TaskToken),
}

impl NotaEncode for ActivityFilter {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::RoleFilter(role) => {
                encoder.start_record("RoleFilter")?;
                role.encode(encoder)?;
                encoder.end_record()
            }
            Self::PathPrefix(path) => {
                encoder.start_record("PathPrefix")?;
                path.encode(encoder)?;
                encoder.end_record()
            }
            Self::TaskToken(token) => {
                encoder.start_record("TaskToken")?;
                token.encode(encoder)?;
                encoder.end_record()
            }
        }
    }
}

impl NotaDecode for ActivityFilter {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        let head = decoder.peek_record_head()?;
        match head.as_str() {
            "RoleFilter" => {
                decoder.expect_record_head("RoleFilter")?;
                let role = RoleName::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::RoleFilter(role))
            }
            "PathPrefix" => {
                decoder.expect_record_head("PathPrefix")?;
                let path = WirePath::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::PathPrefix(path))
            }
            "TaskToken" => {
                decoder.expect_record_head("TaskToken")?;
                let token = TaskToken::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::TaskToken(token))
            }
            other => Err(nota_codec::Error::UnknownVariant {
                enum_name: "ActivityFilter",
                got: other.to_string(),
            }),
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ActivityList {
    /// Ordered most-recent first.
    pub records: Vec<Activity>,
}

// ─── Partial application ──────────────────────────────────

/// Component that participated in a fanned-out orchestration
/// mutation.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum DownstreamComponent {
    Router,
    Harness,
    Terminal,
    Message,
    Mind,
    System,
    Introspect,
}

/// Successful leg of a fanned-out mutation.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ApplicationSuccess {
    pub component: DownstreamComponent,
    pub detail: ScopeReason,
}

/// Typed reason why a downstream leg failed after at least one
/// sibling leg succeeded.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum ApplicationFailureReason {
    Unreachable,
    Rejected,
    Unimplemented,
    TimedOut,
    Unknown,
}

/// Failed leg of a fanned-out mutation.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ApplicationFailure {
    pub component: DownstreamComponent,
    pub reason: ApplicationFailureReason,
    pub detail: ScopeReason,
}

/// Reply when one or more downstream mutation legs were durably
/// applied and one or more sibling legs failed.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PartialApplied {
    pub succeeded: Vec<ApplicationSuccess>,
    pub failed: Vec<ApplicationFailure>,
}

// ─── Observation stream ───────────────────────────────────

/// Subscribe to contract-operation and Sema-effect observations on
/// the public socket.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ObservationSubscription {
    pub include_operations: bool,
    pub include_sema_effects: bool,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaTransparent,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub struct ObservationToken(u64);

impl ObservationToken {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct ObservationOpened {
    pub token: ObservationToken,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct ObservationClosed {
    pub token: ObservationToken,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct OperationReceived {
    pub operation: OperationKind,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct EffectEmitted {
    pub observation: SemaObservation,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum ObservationEvent {
    OperationReceived(OperationReceived),
    EffectEmitted(EffectEmitted),
}

impl NotaEncode for ObservationEvent {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::OperationReceived(observed) => {
                encoder.start_record("OperationReceived")?;
                observed.encode(encoder)?;
                encoder.end_record()
            }
            Self::EffectEmitted(emitted) => {
                encoder.start_record("EffectEmitted")?;
                emitted.encode(encoder)?;
                encoder.end_record()
            }
        }
    }
}

impl NotaDecode for ObservationEvent {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        let head = decoder.peek_record_head()?;
        match head.as_str() {
            "OperationReceived" => {
                decoder.expect_record_head("OperationReceived")?;
                let observed = OperationReceived::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::OperationReceived(observed))
            }
            "EffectEmitted" => {
                decoder.expect_record_head("EffectEmitted")?;
                let emitted = EffectEmitted::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::EffectEmitted(emitted))
            }
            other => Err(nota_codec::Error::UnknownVariant {
                enum_name: "ObservationEvent",
                got: other.to_string(),
            }),
        }
    }
}

// ─── Channel declaration ──────────────────────────────────

signal_channel! {
    channel Orchestrate {
        operation Claim(RoleClaim),
        operation Release(RoleRelease),
        operation Handoff(RoleHandoff),
        operation Observe(Observation),
        operation Submit(ActivitySubmission),
        operation Query(ActivityQuery),
        operation Watch(ObservationSubscription) opens ObservationStream,
        operation Unwatch(ObservationToken),
    }
    reply Reply {
        ClaimAcceptance(ClaimAcceptance),
        ClaimRejection(ClaimRejection),
        ReleaseAcknowledgment(ReleaseAcknowledgment),
        HandoffAcceptance(HandoffAcceptance),
        HandoffRejection(HandoffRejection),
        RoleSnapshot(RoleSnapshot),
        LanesObserved(LanesObserved),
        ActivityAcknowledgment(ActivityAcknowledgment),
        ActivityList(ActivityList),
        PartialApplied(PartialApplied),
        ObservationOpened(ObservationOpened),
        ObservationClosed(ObservationClosed),
    }
    event Event {
        Observed(ObservationEvent) belongs ObservationStream,
    }
    stream ObservationStream {
        token ObservationToken;
        opened ObservationOpened;
        event Observed;
        close Unwatch;
    }
}

pub type OrchestrateRequest = Operation;
pub type OrchestrateReply = Reply;
pub type OrchestrateEvent = Event;
pub type OrchestrateFrame = Frame;
pub type OrchestrateFrameBody = signal_frame::StreamingFrameBody<Operation, Reply, Event>;
pub type OrchestrateChannelRequest = signal_frame::Request<Operation>;
pub type OrchestrateChannelReply = signal_frame::Reply<Reply>;
pub type OrchestrateRequestBuilder = signal_frame::RequestBuilder<Operation>;

impl Operation {
    pub fn operation_kind(&self) -> OperationKind {
        self.kind()
    }
}
