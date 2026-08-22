//! Ordinary Signal contract for atomic Datom path-lock registration.
//!
//! Signal remains binary on the wire. A client converts native Datom values
//! at its text boundary, then sends this contract's rkyv frame.

use datom::{
    PathLockConstructing, PathLockPathConstructing, PathLockPathViewing,
    PathLockRegisteredConstructing, PathLockRegisteredViewing,
    PathLockRegistrationRejectedConstructing, PathLockRegistrationRejectedViewing, PathLockViewing,
};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_frame::{ContractBinding, ContractId, WireContract, WireRevision, signal_channel};

pub use datom::{
    PathLock as NativePathLock, PathLockPath as NativePathLockPath,
    PathLockRegistered as NativePathLockRegistered,
    PathLockRegistrationRejected as NativePathLockRegistrationRejected,
    PathLockRegistrationRejection as NativePathLockRegistrationRejection,
};

/// The binary carrier for the native Datom `PathLock` record.
///
/// It can only be constructed from the native carrier, whose checked
/// constructor validates its name and description and normalizes its nonempty
/// path list.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PathLock {
    name: String,
    paths: Vec<String>,
    description: String,
}

impl TryFrom<NativePathLock> for PathLock {
    type Error = datom::DatomFault;

    fn try_from(value: NativePathLock) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name().into(),
            paths: value.paths().into(),
            description: value.description().into(),
        })
    }
}

impl TryFrom<PathLock> for NativePathLock {
    type Error = datom::DatomFault;

    fn try_from(value: PathLock) -> Result<Self, Self::Error> {
        Self::try_new(value.name, value.paths, value.description)
    }
}

/// The binary carrier for the native Datom normalized conflicting path.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PathLockPath {
    value: String,
}

impl TryFrom<NativePathLockPath> for PathLockPath {
    type Error = datom::DatomFault;

    fn try_from(value: NativePathLockPath) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.path().into(),
        })
    }
}

impl TryFrom<PathLockPath> for NativePathLockPath {
    type Error = datom::DatomFault;

    fn try_from(value: PathLockPath) -> Result<Self, Self::Error> {
        Self::try_new(value.value)
    }
}

/// The ordinary Orchestrate contract's public wire revision.
pub enum OrchestrateWire {}

impl WireContract for OrchestrateWire {
    const BINDING: ContractBinding = ContractBinding::new(
        ContractId::new(core::num::NonZeroU32::MIN),
        WireRevision::new(core::num::NonZeroU16::new(3).expect("literal nonzero revision")),
    );
}

/// A committed, all-or-nothing path-lock registration.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PathLockRegistered {
    pub lock: PathLock,
}

impl TryFrom<NativePathLockRegistered> for PathLockRegistered {
    type Error = datom::DatomFault;

    fn try_from(value: NativePathLockRegistered) -> Result<Self, Self::Error> {
        Ok(Self {
            lock: value.lock().clone().try_into()?,
        })
    }
}

impl TryFrom<PathLockRegistered> for NativePathLockRegistered {
    type Error = datom::DatomFault;

    fn try_from(value: PathLockRegistered) -> Result<Self, Self::Error> {
        Ok(Self::new(value.lock.try_into()?))
    }
}

/// A typed reason why the requested lock was not registered.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum PathLockRegistrationRejection {
    DuplicateActiveName {
        holder: PathLock,
    },
    PathOverlap {
        path: PathLockPath,
        holder: PathLock,
    },
}

impl TryFrom<NativePathLockRegistrationRejection> for PathLockRegistrationRejection {
    type Error = datom::DatomFault;

    fn try_from(value: NativePathLockRegistrationRejection) -> Result<Self, Self::Error> {
        match value {
            NativePathLockRegistrationRejection::DuplicateActiveName { holder } => {
                Ok(Self::DuplicateActiveName {
                    holder: holder.try_into()?,
                })
            }
            NativePathLockRegistrationRejection::PathOverlap { path, holder } => {
                Ok(Self::PathOverlap {
                    path: path.try_into()?,
                    holder: holder.try_into()?,
                })
            }
        }
    }
}

impl TryFrom<PathLockRegistrationRejection> for NativePathLockRegistrationRejection {
    type Error = datom::DatomFault;

    fn try_from(value: PathLockRegistrationRejection) -> Result<Self, Self::Error> {
        match value {
            PathLockRegistrationRejection::DuplicateActiveName { holder } => {
                Ok(Self::DuplicateActiveName {
                    holder: holder.try_into()?,
                })
            }
            PathLockRegistrationRejection::PathOverlap { path, holder } => Ok(Self::PathOverlap {
                path: path.try_into()?,
                holder: holder.try_into()?,
            }),
        }
    }
}

/// A registration refusal. No requested path was registered.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PathLockRegistrationRejected {
    pub requested: PathLock,
    pub reason: PathLockRegistrationRejection,
}

impl TryFrom<NativePathLockRegistrationRejected> for PathLockRegistrationRejected {
    type Error = datom::DatomFault;

    fn try_from(value: NativePathLockRegistrationRejected) -> Result<Self, Self::Error> {
        Ok(Self {
            requested: value.requested().clone().try_into()?,
            reason: value.reason().clone().try_into()?,
        })
    }
}

impl TryFrom<PathLockRegistrationRejected> for NativePathLockRegistrationRejected {
    type Error = datom::DatomFault;

    fn try_from(value: PathLockRegistrationRejected) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value.requested.try_into()?,
            value.reason.try_into()?,
        ))
    }
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
