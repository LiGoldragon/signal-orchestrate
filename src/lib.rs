//! Ordinary Signal contract for atomic Datom path-lock registration.
//!
//! Signal remains binary on the wire. A client converts its Datom
//! `PathLock` at the text boundary, then sends this contract's rkyv frame.

use datom::{EvidencedRealizing, EvidencedTextualizing, ProjectionViewing, RealizationViewing};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_frame::{ContractBinding, ContractId, WireContract, WireRevision, signal_channel};

pub use datom::PathLock as NativePathLock;

/// The binary carrier for the native Datom `PathLock` record.
///
/// It can only be constructed from the native carrier, which canonicalizes
/// its nonempty path list and validates its description.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PathLock {
    name: String,
    paths: Vec<String>,
    description: String,
}

impl TryFrom<NativePathLock> for PathLock {
    type Error = datom::DatomFault;

    fn try_from(value: NativePathLock) -> Result<Self, Self::Error> {
        let projected = value.textualize_evidenced()?;
        let realized = projected.text().realize_evidenced()?;
        let canonical = realized.value();

        Ok(Self {
            name: canonical.name.clone(),
            paths: canonical.paths.clone(),
            description: canonical.description.clone(),
        })
    }
}

impl From<PathLock> for NativePathLock {
    fn from(value: PathLock) -> Self {
        Self {
            name: value.name,
            paths: value.paths,
            description: value.description,
        }
    }
}

/// The ordinary Orchestrate contract's public wire revision.
pub enum OrchestrateWire {}

impl WireContract for OrchestrateWire {
    const BINDING: ContractBinding = ContractBinding::new(
        ContractId::new(core::num::NonZeroU32::MIN),
        WireRevision::new(core::num::NonZeroU16::new(2).expect("literal nonzero revision")),
    );
}

/// A committed, all-or-nothing path-lock registration.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PathLockRegistered {
    pub lock: PathLock,
}

/// A typed reason why the requested lock was not registered.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum PathLockRegistrationRejection {
    DuplicateActiveName { holder: String },
    PathOverlap { path: String, holder: String },
}

/// A registration refusal. No requested path was registered.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PathLockRegistrationRejected {
    pub lock: PathLock,
    pub rejection: PathLockRegistrationRejection,
}

signal_channel! {
    channel Orchestrate contract OrchestrateWire {
        operation Register(PathLock),
    }
    reply Reply {
        PathLockRegistered(PathLockRegistered),
        PathLockRegistrationRejected(PathLockRegistrationRejected),
    }
}

pub type OrchestrateRequest = Operation;
pub type OrchestrateReply = Reply;
pub type OrchestrateFrame = Frame;
pub type OrchestrateFrameBody = FrameBody;
