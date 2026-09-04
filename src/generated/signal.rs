#![allow(dead_code)]
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
pub type LockId = protos::Integer;
pub type LockName = protos::Text;
pub type FlowId = protos::Text;
pub type LockPath = protos::Text;
pub type LockReason = protos::Text;
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockRequest(pub LockName, pub FlowId, pub Vec<LockPath>, pub LockReason);
impl datomic::Corporal<datomic::Datom> for LockRequest {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Struct(fields) if fields.len() == 4usize => {
                let mut iter = fields.into_iter();
                Ok(Self(
                    <LockName as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                    <FlowId as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                    <Vec<LockPath> as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                    <LockReason as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                ))
            }
            datomic::Datom::Struct(fields) => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Arity(4i64, fields.len() as i64),
            )),
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Struct, other),
            )),
        }
    }
}
impl datomic::Datomic for LockRequest {
    fn datomize(&self) -> datomic::Datom {
        datomic::Datom::Struct(vec![
            datomic::Datomic::datomize(&self.0),
            datomic::Datomic::datomize(&self.1),
            datomic::Datomic::datomize(&self.2),
            datomic::Datomic::datomize(&self.3),
        ])
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct Lock(
    pub LockId,
    pub LockName,
    pub FlowId,
    pub Vec<LockPath>,
    pub LockReason,
);
impl datomic::Corporal<datomic::Datom> for Lock {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Struct(fields) if fields.len() == 5usize => {
                let mut iter = fields.into_iter();
                Ok(Self(
                    <LockId as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                    <LockName as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                    <FlowId as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                    <Vec<LockPath> as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                    <LockReason as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                ))
            }
            datomic::Datom::Struct(fields) => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Arity(5i64, fields.len() as i64),
            )),
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Struct, other),
            )),
        }
    }
}
impl datomic::Datomic for Lock {
    fn datomize(&self) -> datomic::Datom {
        datomic::Datom::Struct(vec![
            datomic::Datomic::datomize(&self.0),
            datomic::Datomic::datomize(&self.1),
            datomic::Datomic::datomize(&self.2),
            datomic::Datomic::datomize(&self.3),
            datomic::Datomic::datomize(&self.4),
        ])
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockOverlap(pub LockPath, pub Lock);
impl datomic::Corporal<datomic::Datom> for LockOverlap {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Struct(fields) if fields.len() == 2usize => {
                let mut iter = fields.into_iter();
                Ok(Self(
                    <LockPath as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                    <Lock as datomic::Corporal<datomic::Datom>>::incorporate(iter.next().unwrap())?,
                ))
            }
            datomic::Datom::Struct(fields) => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Arity(2i64, fields.len() as i64),
            )),
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Struct, other),
            )),
        }
    }
}
impl datomic::Datomic for LockOverlap {
    fn datomize(&self) -> datomic::Datom {
        datomic::Datom::Struct(vec![
            datomic::Datomic::datomize(&self.0),
            datomic::Datomic::datomize(&self.1),
        ])
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum LockRejection {
    DuplicateName(Lock),
    PathOverlap(LockOverlap),
}
impl datomic::Corporal<datomic::Datom> for LockRejection {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(DuplicateName) =>
            {
                Ok(Self::DuplicateName(<Lock as datomic::Corporal<
                    datomic::Datom,
                >>::incorporate(*body)?))
            }
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(PathOverlap) =>
            {
                Ok(Self::PathOverlap(<LockOverlap as datomic::Corporal<
                    datomic::Datom,
                >>::incorporate(*body)?))
            }
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, other),
            )),
        }
    }
}
impl datomic::Datomic for LockRejection {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::DuplicateName(value) => datomic::Datom::Variant(
                stringify!(DuplicateName).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
            ),
            Self::PathOverlap(value) => datomic::Datom::Variant(
                stringify!(PathOverlap).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
            ),
        }
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum ReleaseRejection {
    UnknownLockId,
}
impl datomic::Corporal<datomic::Datom> for ReleaseRejection {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Bare(s) if s == stringify!(UnknownLockId) => Ok(Self::UnknownLockId),
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, other),
            )),
        }
    }
}
impl datomic::Datomic for ReleaseRejection {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::UnknownLockId => datomic::Datom::Bare(stringify!(UnknownLockId).to_owned()),
        }
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum ObserveSelection {
    Locks,
}
impl datomic::Corporal<datomic::Datom> for ObserveSelection {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Bare(s) if s == stringify!(Locks) => Ok(Self::Locks),
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, other),
            )),
        }
    }
}
impl datomic::Datomic for ObserveSelection {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::Locks => datomic::Datom::Bare(stringify!(Locks).to_owned()),
        }
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum Observation {
    Locks(Vec<Lock>),
}
impl datomic::Corporal<datomic::Datom> for Observation {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(Locks) =>
            {
                Ok(Self::Locks(<Vec<Lock> as datomic::Corporal<
                    datomic::Datom,
                >>::incorporate(*body)?))
            }
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, other),
            )),
        }
    }
}
impl datomic::Datomic for Observation {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::Locks(value) => datomic::Datom::Variant(
                stringify!(Locks).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
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
impl datomic::Corporal<datomic::Datom> for Request {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(Lock) =>
            {
                Ok(Self::Lock(<LockRequest as datomic::Corporal<
                    datomic::Datom,
                >>::incorporate(*body)?))
            }
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(Release) =>
            {
                Ok(Self::Release(<LockId as datomic::Corporal<
                    datomic::Datom,
                >>::incorporate(*body)?))
            }
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(Observe) =>
            {
                Ok(Self::Observe(<ObserveSelection as datomic::Corporal<
                    datomic::Datom,
                >>::incorporate(*body)?))
            }
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, other),
            )),
        }
    }
}
impl datomic::Datomic for Request {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::Lock(value) => datomic::Datom::Variant(
                stringify!(Lock).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
            ),
            Self::Release(value) => datomic::Datom::Variant(
                stringify!(Release).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
            ),
            Self::Observe(value) => datomic::Datom::Variant(
                stringify!(Observe).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
            ),
        }
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Locked(Lock),
    Released(Lock),
    Observed(Observation),
    LockRejected(LockRejection),
    ReleaseRejected(ReleaseRejection),
}
impl datomic::Corporal<datomic::Datom> for Reply {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(Locked) =>
            {
                Ok(Self::Locked(
                    <Lock as datomic::Corporal<datomic::Datom>>::incorporate(*body)?,
                ))
            }
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(Released) =>
            {
                Ok(Self::Released(<Lock as datomic::Corporal<
                    datomic::Datom,
                >>::incorporate(*body)?))
            }
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(Observed) =>
            {
                Ok(Self::Observed(<Observation as datomic::Corporal<
                    datomic::Datom,
                >>::incorporate(*body)?))
            }
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(LockRejected) =>
            {
                Ok(Self::LockRejected(<LockRejection as datomic::Corporal<
                    datomic::Datom,
                >>::incorporate(*body)?))
            }
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(ReleaseRejected) =>
            {
                Ok(Self::ReleaseRejected(
                    <ReleaseRejection as datomic::Corporal<datomic::Datom>>::incorporate(*body)?,
                ))
            }
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, other),
            )),
        }
    }
}
impl datomic::Datomic for Reply {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::Locked(value) => datomic::Datom::Variant(
                stringify!(Locked).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
            ),
            Self::Released(value) => datomic::Datom::Variant(
                stringify!(Released).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
            ),
            Self::Observed(value) => datomic::Datom::Variant(
                stringify!(Observed).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
            ),
            Self::LockRejected(value) => datomic::Datom::Variant(
                stringify!(LockRejected).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
            ),
            Self::ReleaseRejected(value) => datomic::Datom::Variant(
                stringify!(ReleaseRejected).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
            ),
        }
    }
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Version(pub u16, pub u16, pub u16);
pub const SIGNAL_VERSION: Version = Version(1u16, 0u16, 0u16);
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    VersionMismatch(Version, Version),
    Unreadable,
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum Body {
    Request(Request),
    Reply(Reply),
    Refusal(Refusal),
}
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct Frame(pub Version, pub Body);
impl datomic::Corporal<datomic::Datom> for Version {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Struct(fields) if fields.len() == 3 => {
                let mut it = fields.into_iter();
                let a = <protos::Integer as datomic::Corporal<datomic::Datom>>::incorporate(
                    it.next().unwrap(),
                )? as u16;
                let b = <protos::Integer as datomic::Corporal<datomic::Datom>>::incorporate(
                    it.next().unwrap(),
                )? as u16;
                let c = <protos::Integer as datomic::Corporal<datomic::Datom>>::incorporate(
                    it.next().unwrap(),
                )? as u16;
                Ok(Self(a, b, c))
            }
            datomic::Datom::Struct(fields) => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Arity(3, fields.len() as i64),
            )),
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Struct, other),
            )),
        }
    }
}
impl datomic::Datomic for Version {
    fn datomize(&self) -> datomic::Datom {
        datomic::Datom::Struct(vec![
            datomic::Datomic::datomize(&(self.0 as protos::Integer)),
            datomic::Datomic::datomize(&(self.1 as protos::Integer)),
            datomic::Datomic::datomize(&(self.2 as protos::Integer)),
        ])
    }
}
impl datomic::Corporal<datomic::Datom> for Refusal {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == "VersionMismatch" =>
            {
                match *body {
                    datomic::Datom::Struct(fields) if fields.len() == 2 => {
                        let mut it = fields.into_iter();
                        Ok(Self::VersionMismatch(
                            <Version as datomic::Corporal<datomic::Datom>>::incorporate(
                                it.next().unwrap(),
                            )?,
                            <Version as datomic::Corporal<datomic::Datom>>::incorporate(
                                it.next().unwrap(),
                            )?,
                        ))
                    }
                    other => Err(datomic::Fault::Corporal(
                        vec![],
                        datomic::Problem::Shape(datomic::Expected::Struct, other),
                    )),
                }
            }
            datomic::Datom::Bare(s) if s == "Unreadable" => Ok(Self::Unreadable),
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, other),
            )),
        }
    }
}
impl datomic::Datomic for Refusal {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::VersionMismatch(a, b) => datomic::Datom::Variant(
                "VersionMismatch".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datom::Struct(vec![
                    datomic::Datomic::datomize(a),
                    datomic::Datomic::datomize(b),
                ]))),
            ),
            Self::Unreadable => datomic::Datom::Bare("Unreadable".to_owned()),
        }
    }
}
impl datomic::Corporal<datomic::Datom> for Body {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body)) => {
                match head.as_str() {
                    "Request" => Ok(Self::Request(<Request as datomic::Corporal<
                        datomic::Datom,
                    >>::incorporate(*body)?)),
                    "Reply" => Ok(Self::Reply(
                        <Reply as datomic::Corporal<datomic::Datom>>::incorporate(*body)?,
                    )),
                    "Refusal" => Ok(Self::Refusal(<Refusal as datomic::Corporal<
                        datomic::Datom,
                    >>::incorporate(*body)?)),
                    _ => Err(datomic::Fault::Corporal(
                        vec![],
                        datomic::Problem::UnknownVariant(head),
                    )),
                }
            }
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, other),
            )),
        }
    }
}
impl datomic::Datomic for Body {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::Request(v) => datomic::Datom::Variant(
                "Request".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
            Self::Reply(v) => datomic::Datom::Variant(
                "Reply".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
            Self::Refusal(v) => datomic::Datom::Variant(
                "Refusal".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
        }
    }
}
impl datomic::Corporal<datomic::Datom> for Frame {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Struct(fields) if fields.len() == 2 => {
                let mut it = fields.into_iter();
                Ok(Self(
                    <Version as datomic::Corporal<datomic::Datom>>::incorporate(
                        it.next().unwrap(),
                    )?,
                    <Body as datomic::Corporal<datomic::Datom>>::incorporate(it.next().unwrap())?,
                ))
            }
            datomic::Datom::Struct(fields) => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Arity(2, fields.len() as i64),
            )),
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Struct, other),
            )),
        }
    }
}
impl datomic::Datomic for Frame {
    fn datomize(&self) -> datomic::Datom {
        datomic::Datom::Struct(vec![
            datomic::Datomic::datomize(&self.0),
            datomic::Datomic::datomize(&self.1),
        ])
    }
}
