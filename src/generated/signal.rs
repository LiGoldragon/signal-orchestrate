use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}
impl ProtocolVersion {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelContractId(pub u32);
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelWireRevision(pub u16);
pub const INTERFACE_VERSION: ProtocolVersion = ProtocolVersion::new(0u16, 3u16, 0u16);
pub const CHANNEL_CONTRACT_ID: ChannelContractId = ChannelContractId(1u32);
pub const CHANNEL_WIRE_REVISION: ChannelWireRevision = ChannelWireRevision(6u16);
pub const PROTOCOL_VERSION: ProtocolVersion = INTERFACE_VERSION;
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockName(String);
impl LockName {
    pub fn try_from_string(
        value: String,
    ) -> std::result::Result<Self, datomic::UnrepresentableString> {
        datomic::DatomicString::try_from(value).map(|value| Self(value.as_ref().to_owned()))
    }
}
impl std::convert::TryFrom<String> for LockName {
    type Error = datomic::UnrepresentableString;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::try_from_string(value)
    }
}
impl<'a> std::convert::TryFrom<&'a str> for LockName {
    type Error = datomic::UnrepresentableString;
    fn try_from(value: &'a str) -> std::result::Result<Self, Self::Error> {
        Self::try_from_string(value.to_owned())
    }
}
impl AsRef<str> for LockName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct FlowId(String);
impl FlowId {
    pub fn try_from_string(
        value: String,
    ) -> std::result::Result<Self, datomic::UnrepresentableString> {
        datomic::DatomicString::try_from(value).map(|value| Self(value.as_ref().to_owned()))
    }
}
impl std::convert::TryFrom<String> for FlowId {
    type Error = datomic::UnrepresentableString;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::try_from_string(value)
    }
}
impl<'a> std::convert::TryFrom<&'a str> for FlowId {
    type Error = datomic::UnrepresentableString;
    fn try_from(value: &'a str) -> std::result::Result<Self, Self::Error> {
        Self::try_from_string(value.to_owned())
    }
}
impl AsRef<str> for FlowId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockPath(String);
impl LockPath {
    pub fn try_from_string(
        value: String,
    ) -> std::result::Result<Self, datomic::UnrepresentableString> {
        datomic::DatomicString::try_from(value).map(|value| Self(value.as_ref().to_owned()))
    }
}
impl std::convert::TryFrom<String> for LockPath {
    type Error = datomic::UnrepresentableString;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::try_from_string(value)
    }
}
impl<'a> std::convert::TryFrom<&'a str> for LockPath {
    type Error = datomic::UnrepresentableString;
    fn try_from(value: &'a str) -> std::result::Result<Self, Self::Error> {
        Self::try_from_string(value.to_owned())
    }
}
impl AsRef<str> for LockPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockPaths(pub Vec<LockPath>);
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockReason(String);
impl LockReason {
    pub fn try_from_string(
        value: String,
    ) -> std::result::Result<Self, datomic::UnrepresentableString> {
        datomic::DatomicString::try_from(value).map(|value| Self(value.as_ref().to_owned()))
    }
}
impl std::convert::TryFrom<String> for LockReason {
    type Error = datomic::UnrepresentableString;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::try_from_string(value)
    }
}
impl<'a> std::convert::TryFrom<&'a str> for LockReason {
    type Error = datomic::UnrepresentableString;
    fn try_from(value: &'a str) -> std::result::Result<Self, Self::Error> {
        Self::try_from_string(value.to_owned())
    }
}
impl AsRef<str> for LockReason {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockRequest {
    pub lock_name: LockName,
    pub flow_id: FlowId,
    pub lock_paths: LockPaths,
    pub lock_reason: LockReason,
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockId(pub i64);
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct Lock {
    pub lock_id: LockId,
    pub lock_name: LockName,
    pub flow_id: FlowId,
    pub lock_paths: LockPaths,
    pub lock_reason: LockReason,
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct DuplicateName(pub Lock);
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockOverlap {
    pub lock_path: LockPath,
    pub lock: Lock,
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum LockRejection {
    DuplicateName(Lock),
    PathOverlap(LockOverlap),
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum ReleaseRejection {
    UnknownLockId,
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum ObserveSelection {
    Locks,
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct Locks(pub Vec<Lock>);
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum Observation {
    Locks(Locks),
}
impl datomic::Datomic for LockName {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        Ok(Self(
            <datomic::DatomicString as datomic::Datomic>::embody(portion)?
                .as_ref()
                .to_owned(),
        ))
    }
    fn portion(&self) -> protos::Portion {
        datomic::DatomicString::try_from(self.0.clone()).map_or_else(
            |_| datomic::PortionBuilding::bare("wire-invalid"),
            |value| datomic::Datomic::portion(&value),
        )
    }
}
impl datomic::Datomic for FlowId {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        Ok(Self(
            <datomic::DatomicString as datomic::Datomic>::embody(portion)?
                .as_ref()
                .to_owned(),
        ))
    }
    fn portion(&self) -> protos::Portion {
        datomic::DatomicString::try_from(self.0.clone()).map_or_else(
            |_| datomic::PortionBuilding::bare("wire-invalid"),
            |value| datomic::Datomic::portion(&value),
        )
    }
}
impl datomic::Datomic for LockPath {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        Ok(Self(
            <datomic::DatomicString as datomic::Datomic>::embody(portion)?
                .as_ref()
                .to_owned(),
        ))
    }
    fn portion(&self) -> protos::Portion {
        datomic::DatomicString::try_from(self.0.clone()).map_or_else(
            |_| datomic::PortionBuilding::bare("wire-invalid"),
            |value| datomic::Datomic::portion(&value),
        )
    }
}
impl datomic::Datomic for LockPaths {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        Ok(Self(<Vec<LockPath> as datomic::Datomic>::embody(portion)?))
    }
    fn portion(&self) -> protos::Portion {
        <Vec<LockPath> as datomic::Datomic>::portion(&self.0)
    }
}
impl datomic::Datomic for LockReason {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        Ok(Self(
            <datomic::DatomicString as datomic::Datomic>::embody(portion)?
                .as_ref()
                .to_owned(),
        ))
    }
    fn portion(&self) -> protos::Portion {
        datomic::DatomicString::try_from(self.0.clone()).map_or_else(
            |_| datomic::PortionBuilding::bare("wire-invalid"),
            |value| datomic::Datomic::portion(&value),
        )
    }
}
impl datomic::Datomic for LockRequest {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        let Some(parts) =
            datomic::PortionViewing::structural(portion, protos::StructuralEnclosure::Braced)
        else {
            return Err(datomic::PortionViewing::fault(
                portion,
                datomic::FaultProblem::Shape,
            ));
        };
        if parts.len() != 4usize {
            return Err(datomic::PortionViewing::fault(
                portion,
                datomic::FaultProblem::Arity,
            ));
        }
        Ok(Self {
            lock_name: <LockName as datomic::Datomic>::embody(&parts[0usize])?,
            flow_id: <FlowId as datomic::Datomic>::embody(&parts[1usize])?,
            lock_paths: <LockPaths as datomic::Datomic>::embody(&parts[2usize])?,
            lock_reason: <LockReason as datomic::Datomic>::embody(&parts[3usize])?,
        })
    }
    fn portion(&self) -> protos::Portion {
        datomic::PortionBuilding::structural(
            "",
            protos::StructuralEnclosure::Braced,
            vec![
                datomic::Datomic::portion(&self.lock_name),
                datomic::Datomic::portion(&self.flow_id),
                datomic::Datomic::portion(&self.lock_paths),
                datomic::Datomic::portion(&self.lock_reason),
            ],
        )
    }
}
impl datomic::Datomic for LockId {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        Ok(Self(<i64 as datomic::Datomic>::embody(portion)?))
    }
    fn portion(&self) -> protos::Portion {
        datomic::Datomic::portion(&self.0)
    }
}
impl datomic::Datomic for Lock {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        let Some(parts) =
            datomic::PortionViewing::structural(portion, protos::StructuralEnclosure::Braced)
        else {
            return Err(datomic::PortionViewing::fault(
                portion,
                datomic::FaultProblem::Shape,
            ));
        };
        if parts.len() != 5usize {
            return Err(datomic::PortionViewing::fault(
                portion,
                datomic::FaultProblem::Arity,
            ));
        }
        Ok(Self {
            lock_id: <LockId as datomic::Datomic>::embody(&parts[0usize])?,
            lock_name: <LockName as datomic::Datomic>::embody(&parts[1usize])?,
            flow_id: <FlowId as datomic::Datomic>::embody(&parts[2usize])?,
            lock_paths: <LockPaths as datomic::Datomic>::embody(&parts[3usize])?,
            lock_reason: <LockReason as datomic::Datomic>::embody(&parts[4usize])?,
        })
    }
    fn portion(&self) -> protos::Portion {
        datomic::PortionBuilding::structural(
            "",
            protos::StructuralEnclosure::Braced,
            vec![
                datomic::Datomic::portion(&self.lock_id),
                datomic::Datomic::portion(&self.lock_name),
                datomic::Datomic::portion(&self.flow_id),
                datomic::Datomic::portion(&self.lock_paths),
                datomic::Datomic::portion(&self.lock_reason),
            ],
        )
    }
}
impl datomic::Datomic for DuplicateName {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        Ok(Self(<Lock as datomic::Datomic>::embody(portion)?))
    }
    fn portion(&self) -> protos::Portion {
        <Lock as datomic::Datomic>::portion(&self.0)
    }
}
impl datomic::Datomic for LockOverlap {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        let Some(parts) =
            datomic::PortionViewing::structural(portion, protos::StructuralEnclosure::Braced)
        else {
            return Err(datomic::PortionViewing::fault(
                portion,
                datomic::FaultProblem::Shape,
            ));
        };
        if parts.len() != 2usize {
            return Err(datomic::PortionViewing::fault(
                portion,
                datomic::FaultProblem::Arity,
            ));
        }
        Ok(Self {
            lock_path: <LockPath as datomic::Datomic>::embody(&parts[0usize])?,
            lock: <Lock as datomic::Datomic>::embody(&parts[1usize])?,
        })
    }
    fn portion(&self) -> protos::Portion {
        datomic::PortionBuilding::structural(
            "",
            protos::StructuralEnclosure::Braced,
            vec![
                datomic::Datomic::portion(&self.lock_path),
                datomic::Datomic::portion(&self.lock),
            ],
        )
    }
}
impl datomic::Datomic for LockRejection {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        if let Some(headed) = datomic::PortionViewing::headed(portion)
            && headed.head.as_ref() == stringify!(DuplicateName)
            && headed.separator == protos::Separator::Period
        {
            return Ok(Self::DuplicateName(<Lock as datomic::Datomic>::embody(
                &headed.body,
            )?));
        }
        if let Some(headed) = datomic::PortionViewing::headed(portion)
            && headed.head.as_ref() == stringify!(PathOverlap)
            && headed.separator == protos::Separator::Period
        {
            return Ok(Self::PathOverlap(
                <LockOverlap as datomic::Datomic>::embody(&headed.body)?,
            ));
        }
        Err(datomic::PortionViewing::fault(
            portion,
            datomic::FaultProblem::Shape,
        ))
    }
    fn portion(&self) -> protos::Portion {
        match self {
            Self::DuplicateName(value) => datomic::PortionBuilding::headed(
                stringify!(DuplicateName),
                protos::Separator::Period,
                <Lock as datomic::Datomic>::portion(value),
            ),
            Self::PathOverlap(value) => datomic::PortionBuilding::headed(
                stringify!(PathOverlap),
                protos::Separator::Period,
                <LockOverlap as datomic::Datomic>::portion(value),
            ),
        }
    }
}
impl datomic::Datomic for ReleaseRejection {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        if datomic::PortionViewing::bare_symbol(portion) == Some(stringify!(UnknownLockId)) {
            return Ok(Self::UnknownLockId);
        }
        Err(datomic::PortionViewing::fault(
            portion,
            datomic::FaultProblem::Shape,
        ))
    }
    fn portion(&self) -> protos::Portion {
        match self {
            Self::UnknownLockId => datomic::PortionBuilding::bare(stringify!(UnknownLockId)),
        }
    }
}
impl datomic::Datomic for ObserveSelection {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        if datomic::PortionViewing::bare_symbol(portion) == Some(stringify!(Locks)) {
            return Ok(Self::Locks);
        }
        Err(datomic::PortionViewing::fault(
            portion,
            datomic::FaultProblem::Shape,
        ))
    }
    fn portion(&self) -> protos::Portion {
        match self {
            Self::Locks => datomic::PortionBuilding::bare(stringify!(Locks)),
        }
    }
}
impl datomic::Datomic for Locks {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        Ok(Self(<Vec<Lock> as datomic::Datomic>::embody(portion)?))
    }
    fn portion(&self) -> protos::Portion {
        <Vec<Lock> as datomic::Datomic>::portion(&self.0)
    }
}
impl datomic::Datomic for Observation {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        if let Some(headed) = datomic::PortionViewing::headed(portion)
            && headed.head.as_ref() == stringify!(Locks)
            && headed.separator == protos::Separator::Period
        {
            return Ok(Self::Locks(<Locks as datomic::Datomic>::embody(
                &headed.body,
            )?));
        }
        Err(datomic::PortionViewing::fault(
            portion,
            datomic::FaultProblem::Shape,
        ))
    }
    fn portion(&self) -> protos::Portion {
        match self {
            Self::Locks(value) => datomic::PortionBuilding::headed(
                stringify!(Locks),
                protos::Separator::Period,
                <Locks as datomic::Datomic>::portion(value),
            ),
        }
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Lock(LockRequest),
    Release(LockId),
    Observe(ObserveSelection),
}
impl datomic::Datomic for Request {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        if let Some(headed) = datomic::PortionViewing::headed(portion)
            && headed.head.as_ref() == stringify!(Lock)
            && headed.separator == protos::Separator::Period
        {
            return Ok(Self::Lock(<LockRequest as datomic::Datomic>::embody(
                &headed.body,
            )?));
        }
        if let Some(headed) = datomic::PortionViewing::headed(portion)
            && headed.head.as_ref() == stringify!(Release)
            && headed.separator == protos::Separator::Period
        {
            return Ok(Self::Release(<LockId as datomic::Datomic>::embody(
                &headed.body,
            )?));
        }
        if let Some(headed) = datomic::PortionViewing::headed(portion)
            && headed.head.as_ref() == stringify!(Observe)
            && headed.separator == protos::Separator::Period
        {
            return Ok(Self::Observe(
                <ObserveSelection as datomic::Datomic>::embody(&headed.body)?,
            ));
        }
        Err(datomic::PortionViewing::fault(
            portion,
            datomic::FaultProblem::Shape,
        ))
    }
    fn portion(&self) -> protos::Portion {
        match self {
            Self::Lock(value) => datomic::PortionBuilding::headed(
                stringify!(Lock),
                protos::Separator::Period,
                <LockRequest as datomic::Datomic>::portion(value),
            ),
            Self::Release(value) => datomic::PortionBuilding::headed(
                stringify!(Release),
                protos::Separator::Period,
                <LockId as datomic::Datomic>::portion(value),
            ),
            Self::Observe(value) => datomic::PortionBuilding::headed(
                stringify!(Observe),
                protos::Separator::Period,
                <ObserveSelection as datomic::Datomic>::portion(value),
            ),
        }
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Locked(Lock),
    Released(Lock),
    Observed(Observation),
}
impl datomic::Datomic for Reply {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        if let Some(headed) = datomic::PortionViewing::headed(portion)
            && headed.head.as_ref() == stringify!(Locked)
            && headed.separator == protos::Separator::Period
        {
            return Ok(Self::Locked(<Lock as datomic::Datomic>::embody(
                &headed.body,
            )?));
        }
        if let Some(headed) = datomic::PortionViewing::headed(portion)
            && headed.head.as_ref() == stringify!(Released)
            && headed.separator == protos::Separator::Period
        {
            return Ok(Self::Released(<Lock as datomic::Datomic>::embody(
                &headed.body,
            )?));
        }
        if let Some(headed) = datomic::PortionViewing::headed(portion)
            && headed.head.as_ref() == stringify!(Observed)
            && headed.separator == protos::Separator::Period
        {
            return Ok(Self::Observed(<Observation as datomic::Datomic>::embody(
                &headed.body,
            )?));
        }
        Err(datomic::PortionViewing::fault(
            portion,
            datomic::FaultProblem::Shape,
        ))
    }
    fn portion(&self) -> protos::Portion {
        match self {
            Self::Locked(value) => datomic::PortionBuilding::headed(
                stringify!(Locked),
                protos::Separator::Period,
                <Lock as datomic::Datomic>::portion(value),
            ),
            Self::Released(value) => datomic::PortionBuilding::headed(
                stringify!(Released),
                protos::Separator::Period,
                <Lock as datomic::Datomic>::portion(value),
            ),
            Self::Observed(value) => datomic::PortionBuilding::headed(
                stringify!(Observed),
                protos::Separator::Period,
                <Observation as datomic::Datomic>::portion(value),
            ),
        }
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    LockRejected(LockRejection),
    ReleaseRejected(ReleaseRejection),
}
impl datomic::Datomic for Refusal {
    fn embody(portion: &protos::Portion) -> std::result::Result<Self, datomic::Fault> {
        if let Some(headed) = datomic::PortionViewing::headed(portion)
            && headed.head.as_ref() == stringify!(LockRejected)
            && headed.separator == protos::Separator::Period
        {
            return Ok(Self::LockRejected(
                <LockRejection as datomic::Datomic>::embody(&headed.body)?,
            ));
        }
        if let Some(headed) = datomic::PortionViewing::headed(portion)
            && headed.head.as_ref() == stringify!(ReleaseRejected)
            && headed.separator == protos::Separator::Period
        {
            return Ok(Self::ReleaseRejected(
                <ReleaseRejection as datomic::Datomic>::embody(&headed.body)?,
            ));
        }
        Err(datomic::PortionViewing::fault(
            portion,
            datomic::FaultProblem::Shape,
        ))
    }
    fn portion(&self) -> protos::Portion {
        match self {
            Self::LockRejected(value) => datomic::PortionBuilding::headed(
                stringify!(LockRejected),
                protos::Separator::Period,
                <LockRejection as datomic::Datomic>::portion(value),
            ),
            Self::ReleaseRejected(value) => datomic::PortionBuilding::headed(
                stringify!(ReleaseRejected),
                protos::Separator::Period,
                <ReleaseRejection as datomic::Datomic>::portion(value),
            ),
        }
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum FrameBody {
    Request(Request),
    Reply(Reply),
    Refusal(Refusal),
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub channel_contract_id: ChannelContractId,
    pub channel_wire_revision: ChannelWireRevision,
    pub protocol_version: ProtocolVersion,
    pub body: FrameBody,
}
