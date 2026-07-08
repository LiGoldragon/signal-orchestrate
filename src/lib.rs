//! Signal contract — `orchestrate` CLI ↔ `orchestrate`.
//!
//! Read this file as the public interface of the workspace
//! orchestration channel. The channel carries:
//!
//! - **Role claim/release/handoff** — the claim-flow vocabulary
//!   that the `orchestrate` daemon implements.
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
//! inbound operation kinds and daemon-emitted operation effects.
//!
//! See `ARCHITECTURE.md` for the channel's role and
//! boundaries; `~/primary/skills/contract-repo.md` for the
//! contract-repo discipline this crate follows.

use nota::{Block, Delimiter, NotaBlock, NotaDecode, NotaDecodeError, NotaEncode};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_criome::{
    AuthorizedObjectReference, ContractDigest, EvaluationDecision, ObjectDigest, OperationDigest,
    WorkflowDigest, WorkflowReceipt,
};
use signal_frame::signal_channel;
pub use signal_harness::{
    CapabilityProfile, ClaudeSessionIdentifier, CodexContinuationIdentifier, ContinuationHandle,
    ContinuationRequest, EffortRequest, HarnessName, ModelRequest, ModelResolutionRequest,
    ModelResolved, ModelSelector, ModelUnavailable, ModelUnavailableReason, NamedModel,
    PiContinuationIdentifier,
};
use std::fmt;
use std::str::FromStr;

pub mod schema;

// ─── Error ────────────────────────────────────────────────

pub type ContractResult<T> = std::result::Result<T, Error>;

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
        "session identifier must be CamelCase alphanumeric text with no whitespace or path separators: {session}"
    )]
    InvalidSessionIdentifier { session: String },
    #[error(
        "lane identifier must be non-empty, unbracketed, and contain no whitespace or path separators: {lane}"
    )]
    InvalidLaneIdentifier { lane: String },
    #[error(
        "repository name must be non-empty, unbracketed, and contain no whitespace or path separators: {name}"
    )]
    InvalidRepositoryName { name: String },
    #[error("branch name must be non-empty, unbracketed, and contain no whitespace: {branch}")]
    InvalidBranchName { branch: String },
    #[error(
        "lane name must be non-empty, unbracketed, and contain no whitespace or path separators: {lane}"
    )]
    InvalidLaneName { lane: String },
    #[error("worktree purpose must be non-empty and single-line: {purpose}")]
    InvalidPurposeText { purpose: String },
    #[error("lane details must be non-empty and single-line: {detail}")]
    InvalidLaneDetails { detail: String },
    #[error(
        "workflow run digest must be non-empty, unbracketed, and contain no whitespace: {digest}"
    )]
    InvalidWorkflowRunDigest { digest: String },
    #[error("workflow step name must be non-empty, unbracketed, and contain no whitespace: {name}")]
    InvalidWorkflowStepName { name: String },
    #[error("provider name must be non-empty, unbracketed, and contain no whitespace: {name}")]
    InvalidProviderName { name: String },
    #[error("model name must be non-empty, unbracketed, and contain no whitespace: {name}")]
    InvalidModelName { name: String },
    #[error("host name must be non-empty, unbracketed, and contain no whitespace: {name}")]
    InvalidHostName { name: String },
}

macro_rules! validated_string_nota_codec {
    ($type:ty, $constructor:path) => {
        impl NotaDecode for $type {
            fn from_nota_block(block: &Block) -> std::result::Result<Self, NotaDecodeError> {
                let text = String::from_nota_block(block)?;
                $constructor(text).map_err(|error| NotaDecodeError::Parse(error.to_string()))
            }
        }

        impl NotaEncode for $type {
            fn to_nota(&self) -> String {
                self.as_str().to_owned().to_nota()
            }
        }
    };
}

// ─── Identity ─────────────────────────────────────────────

/// A dynamic workspace role identifier.
///
/// Roles are data now, not enum variants. A role is named by
/// the work context it owns, and new roles can be created at
/// runtime through the owner orchestration surface.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord,
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

    pub fn try_new(role: String) -> ContractResult<Self> {
        Self::from_wire_token(role)
    }

    pub fn from_wire_token(role: impl Into<String>) -> ContractResult<Self> {
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

    fn from_str(role: &str) -> ContractResult<Self> {
        Self::from_wire_token(role)
    }
}

impl TryFrom<String> for RoleIdentifier {
    type Error = Error;

    fn try_from(role: String) -> ContractResult<Self> {
        Self::from_wire_token(role)
    }
}

impl TryFrom<&str> for RoleIdentifier {
    type Error = Error;

    fn try_from(role: &str) -> ContractResult<Self> {
        Self::from_wire_token(role)
    }
}

impl AsRef<str> for RoleIdentifier {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

validated_string_nota_codec!(RoleIdentifier, RoleIdentifier::from_wire_token);

macro_rules! validated_token_type {
    ($type:ident, $constructor:ident, $error:ident, $field:ident) => {
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
        )]
        pub struct $type(String);

        impl $type {
            pub fn try_new(value: String) -> ContractResult<Self> {
                Self::$constructor(value)
            }

            pub fn $constructor(value: impl Into<String>) -> ContractResult<Self> {
                let value = value.into();
                if value.is_empty()
                    || value.chars().any(char::is_whitespace)
                    || value.contains('[')
                    || value.contains(']')
                {
                    return Err(Error::$error { $field: value });
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $type {
            type Error = Error;

            fn try_from(value: String) -> ContractResult<Self> {
                Self::$constructor(value)
            }
        }

        impl TryFrom<&str> for $type {
            type Error = Error;

            fn try_from(value: &str) -> ContractResult<Self> {
                Self::$constructor(value)
            }
        }

        impl AsRef<str> for $type {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        validated_string_nota_codec!($type, $type::$constructor);
    };
}

/// One token in a role vector.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub struct RoleToken(String);

impl RoleToken {
    pub fn try_new(token: String) -> ContractResult<Self> {
        Self::from_text(token)
    }

    pub fn from_text(token: impl Into<String>) -> ContractResult<Self> {
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

    fn try_from(token: String) -> ContractResult<Self> {
        Self::from_text(token)
    }
}

impl TryFrom<&str> for RoleToken {
    type Error = Error;

    fn try_from(token: &str) -> ContractResult<Self> {
        Self::from_text(token)
    }
}

impl AsRef<str> for RoleToken {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

validated_string_nota_codec!(RoleToken, RoleToken::from_text);

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct Role {
    pub tokens: Vec<RoleToken>,
}

impl Role {
    pub fn try_new(tokens: Vec<RoleToken>) -> ContractResult<Self> {
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
    Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub struct SessionIdentifier(String);

pub type SessionName = SessionIdentifier;

impl SessionIdentifier {
    pub fn try_new(session: String) -> ContractResult<Self> {
        Self::from_camel_case_name(session)
    }

    pub fn from_camel_case_name(session: impl Into<String>) -> ContractResult<Self> {
        let session = session.into();
        let mut characters = session.chars();
        let Some(first) = characters.next() else {
            return Err(Error::InvalidSessionIdentifier { session });
        };
        if !first.is_ascii_uppercase()
            || !characters.all(|character| character.is_ascii_alphanumeric())
        {
            return Err(Error::InvalidSessionIdentifier { session });
        }
        Ok(Self(session))
    }

    pub fn as_wire_token(&self) -> &str {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionIdentifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_wire_token())
    }
}

impl FromStr for SessionIdentifier {
    type Err = Error;

    fn from_str(session: &str) -> ContractResult<Self> {
        Self::from_camel_case_name(session)
    }
}

impl TryFrom<String> for SessionIdentifier {
    type Error = Error;

    fn try_from(session: String) -> ContractResult<Self> {
        Self::from_camel_case_name(session)
    }
}

impl TryFrom<&str> for SessionIdentifier {
    type Error = Error;

    fn try_from(session: &str) -> ContractResult<Self> {
        Self::from_camel_case_name(session)
    }
}

impl AsRef<str> for SessionIdentifier {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

validated_string_nota_codec!(SessionIdentifier, SessionIdentifier::from_camel_case_name);

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum LaneAuthority {
    Structural,
    Support,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub struct LaneIdentifier(String);

impl LaneIdentifier {
    pub fn try_new(lane: String) -> ContractResult<Self> {
        Self::from_wire_token(lane)
    }

    pub fn from_wire_token(lane: impl Into<String>) -> ContractResult<Self> {
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

    fn try_from(lane: String) -> ContractResult<Self> {
        Self::from_wire_token(lane)
    }
}

impl TryFrom<&str> for LaneIdentifier {
    type Error = Error;

    fn try_from(lane: &str) -> ContractResult<Self> {
        Self::from_wire_token(lane)
    }
}

impl AsRef<str> for LaneIdentifier {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

validated_string_nota_codec!(LaneIdentifier, LaneIdentifier::from_wire_token);

// ─── Worktree identity ────────────────────────────────────

/// The repository a worktree belongs to — the `<repo>` segment
/// under `~/wt/github.com/LiGoldragon/<repo>/<name>`. Same shape
/// as the git-index repository name (`StoredRepository::name`).
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub struct RepositoryName(String);

impl RepositoryName {
    pub fn try_new(name: String) -> ContractResult<Self> {
        Self::from_text(name)
    }

    pub fn from_text(name: impl Into<String>) -> ContractResult<Self> {
        let name = name.into();
        if name.is_empty()
            || name.chars().any(char::is_whitespace)
            || name.contains('/')
            || name.contains('\\')
            || name.contains('[')
            || name.contains(']')
        {
            return Err(Error::InvalidRepositoryName { name });
        }
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RepositoryName {
    type Error = Error;

    fn try_from(name: String) -> ContractResult<Self> {
        Self::from_text(name)
    }
}

impl TryFrom<&str> for RepositoryName {
    type Error = Error;

    fn try_from(name: &str) -> ContractResult<Self> {
        Self::from_text(name)
    }
}

impl AsRef<str> for RepositoryName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

validated_string_nota_codec!(RepositoryName, RepositoryName::from_text);

/// The feature/`next` branch a worktree carries — the `<name>`
/// segment of the worktree path and the jj bookmark name.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub struct BranchName(String);

impl BranchName {
    pub fn try_new(branch: String) -> ContractResult<Self> {
        Self::from_text(branch)
    }

    pub fn from_text(branch: impl Into<String>) -> ContractResult<Self> {
        let branch = branch.into();
        if branch.is_empty()
            || branch.chars().any(char::is_whitespace)
            || branch.contains('[')
            || branch.contains(']')
        {
            return Err(Error::InvalidBranchName { branch });
        }
        Ok(Self(branch))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for BranchName {
    type Error = Error;

    fn try_from(branch: String) -> ContractResult<Self> {
        Self::from_text(branch)
    }
}

impl TryFrom<&str> for BranchName {
    type Error = Error;

    fn try_from(branch: &str) -> ContractResult<Self> {
        Self::from_text(branch)
    }
}

impl AsRef<str> for BranchName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

validated_string_nota_codec!(BranchName, BranchName::from_text);

/// The lane that owns a worktree (the harness window's exact
/// role-name, e.g. `designer`, `second-operator`). Same shape
/// as [`LaneIdentifier`] but named for its worktree-ownership
/// role.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub struct LaneName(String);

impl LaneName {
    pub fn try_new(lane: String) -> ContractResult<Self> {
        Self::from_text(lane)
    }

    pub fn from_text(lane: impl Into<String>) -> ContractResult<Self> {
        let lane = lane.into();
        if lane.is_empty()
            || lane.chars().any(char::is_whitespace)
            || lane.contains('/')
            || lane.contains('\\')
            || lane.contains('[')
            || lane.contains(']')
        {
            return Err(Error::InvalidLaneName { lane });
        }
        Ok(Self(lane))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for LaneName {
    type Error = Error;

    fn try_from(lane: String) -> ContractResult<Self> {
        Self::from_text(lane)
    }
}

impl TryFrom<&str> for LaneName {
    type Error = Error;

    fn try_from(lane: &str) -> ContractResult<Self> {
        Self::from_text(lane)
    }
}

impl AsRef<str> for LaneName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

validated_string_nota_codec!(LaneName, LaneName::from_text);

/// Free-text purpose of a worktree — what the branch is for.
/// Single-line like [`ScopeReason`].
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PurposeText(String);

impl PurposeText {
    pub fn try_new(purpose: String) -> ContractResult<Self> {
        Self::from_text(purpose)
    }

    pub fn from_text(purpose: impl Into<String>) -> ContractResult<Self> {
        let purpose = purpose.into();
        if purpose.is_empty() || purpose.contains('\n') || purpose.contains('\r') {
            return Err(Error::InvalidPurposeText { purpose });
        }
        Ok(Self(purpose))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PurposeText {
    type Error = Error;

    fn try_from(purpose: String) -> ContractResult<Self> {
        Self::from_text(purpose)
    }
}

impl TryFrom<&str> for PurposeText {
    type Error = Error;

    fn try_from(purpose: &str) -> ContractResult<Self> {
        Self::from_text(purpose)
    }
}

impl AsRef<str> for PurposeText {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

validated_string_nota_codec!(PurposeText, PurposeText::from_text);

/// Worktree lifecycle state. `Active` while in use; `Merged`
/// once integrated; `Archived` retained as a GC-manifest record;
/// `Recycled` when the worktree slot was reclaimed.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum WorktreeStatus {
    Active,
    Merged,
    Archived,
    Recycled,
}

/// How a worktree's branch relates to its remote and to `main`.
/// `Unpushed` — local-only, no remote tracking; `Pushed` — has a
/// real remote ref; `AncestorOfMain` — already an ancestor of
/// `main` (merge complete, safe to GC). Derived by the daemon
/// scanner from `jj`, never agent-supplied.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum PushedState {
    Unpushed,
    Pushed,
    AncestorOfMain,
}

/// One registered worktree. `last_activity` is store/scanner
/// supplied (the worktree's newest commit time), never
/// agent-supplied. The `(repository, branch)` pair is the
/// identity.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct Worktree {
    pub repository: RepositoryName,
    pub branch: BranchName,
    pub path: WirePath,
    pub owning_lane: LaneName,
    pub status: WorktreeStatus,
    pub purpose: PurposeText,
    pub last_activity: TimestampNanos,
    pub pushed_state: PushedState,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LaneDetails(String);

impl LaneDetails {
    pub fn try_new(detail: String) -> ContractResult<Self> {
        Self::from_text(detail)
    }

    pub fn from_text(detail: impl Into<String>) -> ContractResult<Self> {
        let detail = detail.into();
        if detail.trim().is_empty() || detail.contains('\n') || detail.contains('\r') {
            return Err(Error::InvalidLaneDetails { detail });
        }
        Ok(Self(detail))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for LaneDetails {
    type Error = Error;

    fn try_from(detail: String) -> ContractResult<Self> {
        Self::from_text(detail)
    }
}

impl TryFrom<&str> for LaneDetails {
    type Error = Error;

    fn try_from(detail: &str) -> ContractResult<Self> {
        Self::from_text(detail)
    }
}

impl AsRef<str> for LaneDetails {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

validated_string_nota_codec!(LaneDetails, LaneDetails::from_text);

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum LaneStatus {
    Active,
    Released,
    HandoverEnded,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct LaneOwner {
    pub role: Role,
    pub authority: LaneAuthority,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct LaneAssignment {
    pub session: SessionIdentifier,
    pub lane: LaneIdentifier,
    pub owner: LaneOwner,
    pub details: LaneDetails,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct LaneRegistration {
    pub assignment: LaneAssignment,
    pub registered_at: TimestampNanos,
    pub status: LaneStatus,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
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

    pub fn from_wire_token(harness: impl Into<String>) -> ContractResult<Self> {
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
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
)]
pub enum ScopeReference {
    /// An absolute file or directory path.
    Path(WirePath),
    /// A bracketed task token like `[primary-f99]` (stored
    /// without brackets here).
    Task(TaskToken),
}

/// Absolute path, newtyped for cross-platform stability on
/// the wire (per `~/primary/skills/rust-discipline.md`
/// §"Newtype the wire form" — `PathBuf` archives
/// non-deterministically).
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WirePath(String);

impl WirePath {
    pub fn try_new(path: String) -> ContractResult<Self> {
        Self::from_absolute_path(path)
    }

    pub fn from_absolute_path(path: impl Into<String>) -> ContractResult<Self> {
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

    fn try_from(path: String) -> ContractResult<Self> {
        Self::from_absolute_path(path)
    }
}

impl TryFrom<&str> for WirePath {
    type Error = Error;

    fn try_from(path: &str) -> ContractResult<Self> {
        Self::from_absolute_path(path)
    }
}

impl AsRef<str> for WirePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

validated_string_nota_codec!(WirePath, WirePath::from_absolute_path);

/// A bracketed task identifier (stored without brackets).
/// Bracketed form like `[primary-f99]` is the human surface;
/// the wire carries the raw token.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskToken(String);

impl TaskToken {
    pub fn try_new(token: String) -> ContractResult<Self> {
        Self::from_wire_token(token)
    }

    pub fn from_wire_token(token: impl Into<String>) -> ContractResult<Self> {
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

    fn try_from(token: String) -> ContractResult<Self> {
        Self::from_wire_token(token)
    }
}

impl TryFrom<&str> for TaskToken {
    type Error = Error;

    fn try_from(token: &str) -> ContractResult<Self> {
        Self::from_wire_token(token)
    }
}

impl AsRef<str> for TaskToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

validated_string_nota_codec!(TaskToken, TaskToken::from_wire_token);

// ─── Reason ───────────────────────────────────────────────

/// A short reason string. Provisional until the typed Nexus record
/// shape for "intent" is named.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopeReason(String);

impl ScopeReason {
    pub fn try_new(reason: String) -> ContractResult<Self> {
        Self::from_text(reason)
    }

    pub fn from_text(reason: impl Into<String>) -> ContractResult<Self> {
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

    fn try_from(reason: String) -> ContractResult<Self> {
        Self::from_text(reason)
    }
}

impl TryFrom<&str> for ScopeReason {
    type Error = Error;

    fn try_from(reason: &str) -> ContractResult<Self> {
        Self::from_text(reason)
    }
}

impl AsRef<str> for ScopeReason {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

validated_string_nota_codec!(ScopeReason, ScopeReason::from_text);

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
    NotaEncode,
    NotaDecode,
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

/// Passive elapsed age in nanoseconds. Age is evidence in observation
/// replies, not a heartbeat or expiry decision.
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
    NotaEncode,
    NotaDecode,
)]
pub struct DurationNanos(u64);

impl DurationNanos {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

// ─── Workflow execution ───────────────────────────────────

validated_token_type!(
    WorkflowRunDigest,
    from_wire_token,
    InvalidWorkflowRunDigest,
    digest
);
validated_token_type!(
    WorkflowStepName,
    from_wire_token,
    InvalidWorkflowStepName,
    name
);
validated_token_type!(ProviderName, from_wire_token, InvalidProviderName, name);
validated_token_type!(ModelName, from_wire_token, InvalidModelName, name);
validated_token_type!(HostName, from_wire_token, InvalidHostName, name);

/// Request to run one content-addressed adjudication workflow for one
/// content-addressed operation under one criome contract.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowRunRequest {
    pub workflow: WorkflowDigest,
    pub operation: AuthorizedObjectReference,
    pub contract: ContractDigest,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ResolvedWorkflowRunRequest {
    pub workflow_run: WorkflowRunRequest,
    pub model_resolution: ModelResolutionRequest,
}

/// Subscribe to one workflow run's lifecycle.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowRunObservation {
    pub run: WorkflowRunDigest,
}

/// Close one workflow-run subscription.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowRunObservationToken {
    pub run: WorkflowRunDigest,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowRunHandle {
    pub run: WorkflowRunDigest,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowRunAccepted {
    pub handle: WorkflowRunHandle,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowRunResolution {
    pub handle: WorkflowRunHandle,
    pub resolution: ModelResolved,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowResolutionUnavailable {
    pub handle: WorkflowRunHandle,
    pub request: ResolvedWorkflowRunRequest,
    pub unavailable: ModelUnavailable,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowReceiptProduced {
    pub handle: WorkflowRunHandle,
    pub receipt: WorkflowReceipt,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowResolvedReceiptProduced {
    pub run: WorkflowRunResolution,
    pub receipt: WorkflowReceipt,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowRunLogReported {
    pub log: WorkflowRunLog,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowRunLog {
    pub run: WorkflowRunDigest,
    pub step_logs: Vec<StepLog>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct StepLog {
    pub step: WorkflowStepName,
    pub attestation: ModelAttestation,
    pub outcome: StepOutcome,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ModelAttestation {
    pub provider: ProviderName,
    pub model: ModelName,
    pub host: HostName,
    pub call: OperationDigest,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub enum StepOutcome {
    Produced(EvaluationDecision),
    Failed(ScopeReason),
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowDefinition {
    pub steps: Vec<WorkflowStep>,
    pub combination: CombinationRule,
    pub escalation: Option<signal_criome::EscalationTarget>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowStep {
    pub name: WorkflowStepName,
    pub prompt: ObjectDigest,
    pub provider: Option<ProviderName>,
    pub dependencies: Vec<WorkflowStepName>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub enum CombinationRule {
    Threshold(StepThreshold),
    Unanimous,
    AnyApprove,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub struct StepThreshold(u64);

impl StepThreshold {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowRunSnapshot {
    pub handle: WorkflowRunHandle,
    pub latest_log: Option<WorkflowRunLog>,
    pub receipt: Option<WorkflowReceipt>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowRunObservationOpened {
    pub token: WorkflowRunObservationToken,
    pub snapshot: WorkflowRunSnapshot,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowRunObservationClosed {
    pub token: WorkflowRunObservationToken,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorkflowRunUpdate {
    pub run: WorkflowRunDigest,
    pub log: Option<WorkflowRunLog>,
    pub receipt: Option<WorkflowReceipt>,
}

// ─── Claim verbs ──────────────────────────────────────────

/// A role asks to claim one or more scopes with a short
/// reason. Reply: `ClaimAcceptance` on success, `ClaimRejection`
/// listing every conflict on failure.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct RoleClaim {
    pub role: RoleName,
    pub scopes: Vec<ScopeReference>,
    pub reason: ScopeReason,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ClaimAcceptance {
    pub role: RoleName,
    pub scopes: Vec<ScopeReference>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ClaimRejection {
    pub role: RoleName,
    pub conflicts: Vec<ScopeConflict>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ScopeConflict {
    pub scope: ScopeReference,
    pub held_by: RoleName,
    pub held_reason: ScopeReason,
}

// ─── Release verbs ────────────────────────────────────────

/// A role releases all of its currently-held scopes.
/// Reply: `ReleaseAcknowledgment` listing what was released.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct RoleRelease {
    pub role: RoleName,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ReleaseAcknowledgment {
    pub role: RoleName,
    pub released_scopes: Vec<ScopeReference>,
}

// ─── Handoff verbs ────────────────────────────────────────

/// One role hands a set of scopes to another role atomically.
/// Reply: `HandoffAcceptance` on success, `HandoffRejection`
/// with a typed reason on failure.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct RoleHandoff {
    pub from: RoleName,
    pub to: RoleName,
    pub scopes: Vec<ScopeReference>,
    pub reason: ScopeReason,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct HandoffAcceptance {
    pub from: RoleName,
    pub to: RoleName,
    pub scopes: Vec<ScopeReference>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct HandoffRejection {
    pub from: RoleName,
    pub to: RoleName,
    pub reason: HandoffRejectionReason,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub enum HandoffRejectionReason {
    /// The `from` role doesn't currently hold the named scopes.
    SourceRoleDoesNotHold,
    /// The `to` role's existing claims conflict with the
    /// scopes being handed off (the conflict list names which
    /// scopes and which existing holders).
    TargetRoleConflict(Vec<ScopeConflict>),
}

// ─── Observation ──────────────────────────────────────────

/// Select what the `Observe` operation reads.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub enum Observation {
    Roles,
    Sessions,
    SessionLanes(SessionIdentifier),
    Lanes,
    Worktrees,
}

/// Legacy empty payload kept for older callers while the `Observe`
/// operation moves to [`Observation`].
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct RoleObservation {}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct RoleSnapshot {
    pub roles: Vec<RoleStatus>,
    pub recent_activity: Vec<Activity>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct SessionsObserved {
    pub sessions: Vec<SessionProjection>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct LanesObserved {
    pub lanes: Vec<LaneProjection>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct SessionProjection {
    pub session: SessionIdentifier,
    pub active_lanes: u64,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct LaneProjection {
    pub registration: LaneRegistration,
    pub resource_claims: Vec<LaneResourceClaim>,
    pub observed_at: TimestampNanos,
    pub age: DurationNanos,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct LaneResourceClaim {
    pub scope: ScopeReference,
    pub reason: ScopeReason,
    pub claimed_at: TimestampNanos,
    pub age: DurationNanos,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct WorktreesObserved {
    pub worktrees: Vec<Worktree>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct RoleStatus {
    pub role: RoleName,
    pub harness: HarnessKind,
    pub claims: Vec<ClaimEntry>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ClaimEntry {
    pub scope: ScopeReference,
    pub reason: ScopeReason,
    pub claimed_at: TimestampNanos,
    pub age: DurationNanos,
}

// ─── Activity log ─────────────────────────────────────────

/// One activity record: who touched what and why. Time is
/// store-supplied (per ESSENCE infrastructure-mints rule —
/// the agent never invents timestamps).
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct Activity {
    pub role: RoleName,
    pub scope: ScopeReference,
    pub reason: ScopeReason,
    pub stamped_at: TimestampNanos,
}

/// Submit a new activity record. The store assigns
/// `stamped_at` on commit. Reply: `ActivityAcknowledgment`
/// carrying the slot the record landed in.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ActivitySubmission {
    pub role: RoleName,
    pub scope: ScopeReference,
    pub reason: ScopeReason,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct ActivityAcknowledgment {
    /// The slot (sequential u64) the record was assigned.
    pub slot: u64,
}

/// Query the activity log. Limit caps how many records come
/// back; filters narrow by role or scope. Empty filter list
/// = "all".
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct ActivityQuery {
    pub limit: u32,
    pub filters: Vec<ActivityFilter>,
}

impl NotaDecode for ActivityQuery {
    fn from_nota_block(block: &Block) -> std::result::Result<Self, NotaDecodeError> {
        let children =
            NotaBlock::new(block).expect_children(Delimiter::Parenthesis, "ActivityQuery", 2)?;
        let limit = u32::try_from(u64::from_nota_block(&children[0])?)
            .map_err(|error| NotaDecodeError::Parse(error.to_string()))?;
        let filters = Vec::<ActivityFilter>::from_nota_block(&children[1])?;
        Ok(Self { limit, filters })
    }
}

impl NotaEncode for ActivityQuery {
    fn to_nota(&self) -> String {
        Delimiter::Parenthesis.wrap([u64::from(self.limit).to_nota(), self.filters.to_nota()])
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
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

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ActivityList {
    /// Ordered most-recent first.
    pub records: Vec<Activity>,
}

// ─── Partial application ──────────────────────────────────

/// Component that participated in a fanned-out orchestration
/// mutation.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
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
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ApplicationSuccess {
    pub component: DownstreamComponent,
    pub detail: ScopeReason,
}

/// Typed reason why a downstream leg failed after at least one
/// sibling leg succeeded.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum ApplicationFailureReason {
    Unreachable,
    Rejected,
    Unimplemented,
    TimedOut,
    Unknown,
}

/// Failed leg of a fanned-out mutation.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ApplicationFailure {
    pub component: DownstreamComponent,
    pub reason: ApplicationFailureReason,
    pub detail: ScopeReason,
}

/// Reply when one or more downstream mutation legs were durably
/// applied and one or more sibling legs failed.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct PartialApplied {
    pub succeeded: Vec<ApplicationSuccess>,
    pub failed: Vec<ApplicationFailure>,
}

// ─── Observation stream ───────────────────────────────────

/// Subscribe to contract-operation and effect observations on
/// the public socket.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ObservationSubscription {
    pub include_operations: bool,
    pub include_effects: bool,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
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
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct ObservationOpened {
    pub token: ObservationToken,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub struct ObservationClosed {
    pub token: ObservationToken,
}

#[cfg_attr(feature = "nota-text", derive(NotaEncode, NotaDecode))]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct OperationReceived {
    pub operation: OperationKind,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum EffectOutcome {
    Applied,
    Removed,
    Changed,
    Observed,
    StreamOpened,
    StreamClosed,
    NoChange,
}

#[cfg_attr(feature = "nota-text", derive(NotaEncode, NotaDecode))]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectEmitted {
    pub operation: OperationKind,
    pub outcome: EffectOutcome,
}

#[cfg_attr(feature = "nota-text", derive(NotaEncode, NotaDecode))]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum ObservationEvent {
    OperationReceived(OperationReceived),
    EffectEmitted(EffectEmitted),
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
        operation RunWorkflow(WorkflowRunRequest),
        operation RunResolvedWorkflow(ResolvedWorkflowRunRequest),
        operation ObserveWorkflowRun(WorkflowRunObservation) opens WorkflowRunStream,
        operation WorkflowRunObservationRetraction(WorkflowRunObservationToken),
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
        SessionsObserved(SessionsObserved),
        LanesObserved(LanesObserved),
        WorktreesObserved(WorktreesObserved),
        ActivityAcknowledgment(ActivityAcknowledgment),
        ActivityList(ActivityList),
        WorkflowRunAccepted(WorkflowRunAccepted),
        WorkflowResolutionAccepted(WorkflowRunResolution),
        WorkflowResolutionUnavailable(WorkflowResolutionUnavailable),
        WorkflowReceiptProduced(WorkflowReceiptProduced),
        WorkflowResolvedReceiptProduced(WorkflowResolvedReceiptProduced),
        WorkflowRunLogReported(WorkflowRunLogReported),
        WorkflowRunObservationOpened(WorkflowRunObservationOpened),
        WorkflowRunObservationClosed(WorkflowRunObservationClosed),
        PartialApplied(PartialApplied),
        ObservationOpened(ObservationOpened),
        ObservationClosed(ObservationClosed),
    }
    event Event {
        WorkflowRunUpdated(WorkflowRunUpdate) belongs WorkflowRunStream,
        Observed(ObservationEvent) belongs ObservationStream,
    }
    stream WorkflowRunStream {
        token WorkflowRunObservationToken;
        opened WorkflowRunObservationOpened;
        event WorkflowRunUpdated;
        close WorkflowRunObservationRetraction;
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
