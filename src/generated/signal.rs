#![allow(dead_code)]
#![allow(clippy::redundant_closure)]
pub type LockId = protos::Integer;
pub type LockName = protos::Text;
pub type FlowId = protos::Text;
pub type LockPath = protos::Text;
pub type LockReason = protos::Text;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockRequest(
    pub LockName,
    pub FlowId,
    pub std::vec::Vec<LockPath>,
    pub LockReason,
);
impl datom_codec::Datomic for LockRequest {
    fn incorporate(site: datom_codec::Site<'_>) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 4)?;
        let p0: LockName = datom_codec::Positional::position(&mut p)?;
        let p1: FlowId = datom_codec::Positional::position(&mut p)?;
        let p2: std::vec::Vec<LockPath> = datom_codec::Positional::position(&mut p)?;
        let p3: LockReason = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0, p1, p2, p3))
    }
}
impl protos::Conceivable<datom_codec::Datom> for LockRequest {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(protos::Situated(
            protos::Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            datom_codec::Datom::Struct(vec![
                protos::Conceivable::conceive(&self.0)
                    .expect("infallible datom ascent")
                    .1,
                protos::Conceivable::conceive(&self.1)
                    .expect("infallible datom ascent")
                    .1,
                protos::Conceivable::conceive(&self.2)
                    .expect("infallible datom ascent")
                    .1,
                protos::Conceivable::conceive(&self.3)
                    .expect("infallible datom ascent")
                    .1,
            ]),
        ))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lock(
    pub LockId,
    pub LockName,
    pub FlowId,
    pub std::vec::Vec<LockPath>,
    pub LockReason,
);
impl datom_codec::Datomic for Lock {
    fn incorporate(site: datom_codec::Site<'_>) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 5)?;
        let p0: LockId = datom_codec::Positional::position(&mut p)?;
        let p1: LockName = datom_codec::Positional::position(&mut p)?;
        let p2: FlowId = datom_codec::Positional::position(&mut p)?;
        let p3: std::vec::Vec<LockPath> = datom_codec::Positional::position(&mut p)?;
        let p4: LockReason = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0, p1, p2, p3, p4))
    }
}
impl protos::Conceivable<datom_codec::Datom> for Lock {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(protos::Situated(
            protos::Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            datom_codec::Datom::Struct(vec![
                protos::Conceivable::conceive(&self.0)
                    .expect("infallible datom ascent")
                    .1,
                protos::Conceivable::conceive(&self.1)
                    .expect("infallible datom ascent")
                    .1,
                protos::Conceivable::conceive(&self.2)
                    .expect("infallible datom ascent")
                    .1,
                protos::Conceivable::conceive(&self.3)
                    .expect("infallible datom ascent")
                    .1,
                protos::Conceivable::conceive(&self.4)
                    .expect("infallible datom ascent")
                    .1,
            ]),
        ))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockOverlap(pub LockPath, pub Lock);
impl datom_codec::Datomic for LockOverlap {
    fn incorporate(site: datom_codec::Site<'_>) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: LockPath = datom_codec::Positional::position(&mut p)?;
        let p1: Lock = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0, p1))
    }
}
impl protos::Conceivable<datom_codec::Datom> for LockOverlap {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(protos::Situated(
            protos::Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            datom_codec::Datom::Struct(vec![
                protos::Conceivable::conceive(&self.0)
                    .expect("infallible datom ascent")
                    .1,
                protos::Conceivable::conceive(&self.1)
                    .expect("infallible datom ascent")
                    .1,
            ]),
        ))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LockRejection {
    DuplicateName(Lock),
    PathOverlap(LockOverlap),
}
impl datom_codec::Datomic for LockRejection {
    fn incorporate(site: datom_codec::Site<'_>) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "DuplicateName" => {
                std::result::Result::Ok(Self::DuplicateName(datom_codec::Carrying::body(v)?))
            }
            "PathOverlap" => {
                std::result::Result::Ok(Self::PathOverlap(datom_codec::Carrying::body(v)?))
            }
            _ => std::result::Result::Err(datom_codec::Headed::reject(
                &v,
                datom_codec::Problem::UnknownVariant(
                    protos::Word::try_from(v.name).expect("variant name"),
                ),
            )),
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for LockRejection {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(protos::Situated(
            protos::Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            match self {
                Self::DuplicateName(p0) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("DuplicateName").expect("static variant"),
                    std::boxed::Box::new(
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                    ),
                ),
                Self::PathOverlap(p0) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("PathOverlap").expect("static variant"),
                    std::boxed::Box::new(
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                    ),
                ),
            },
        ))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseRejection {
    UnknownLockId,
}
impl datom_codec::Datomic for ReleaseRejection {
    fn incorporate(site: datom_codec::Site<'_>) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "UnknownLockId" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::UnknownLockId)
            }
            _ => std::result::Result::Err(datom_codec::Headed::reject(
                &v,
                datom_codec::Problem::UnknownVariant(
                    protos::Word::try_from(v.name).expect("variant name"),
                ),
            )),
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for ReleaseRejection {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(protos::Situated(
            protos::Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            match self {
                Self::UnknownLockId => datom_codec::Datom::Word(
                    datom_codec::DatomWord::try_from(
                        protos::Word::try_from("UnknownLockId").expect("static variant"),
                    )
                    .expect("stable variant"),
                ),
            },
        ))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserveSelection {
    Locks,
}
impl datom_codec::Datomic for ObserveSelection {
    fn incorporate(site: datom_codec::Site<'_>) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Locks" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Locks)
            }
            _ => std::result::Result::Err(datom_codec::Headed::reject(
                &v,
                datom_codec::Problem::UnknownVariant(
                    protos::Word::try_from(v.name).expect("variant name"),
                ),
            )),
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for ObserveSelection {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(protos::Situated(
            protos::Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            match self {
                Self::Locks => datom_codec::Datom::Word(
                    datom_codec::DatomWord::try_from(
                        protos::Word::try_from("Locks").expect("static variant"),
                    )
                    .expect("stable variant"),
                ),
            },
        ))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Observation {
    Locks(std::vec::Vec<Lock>),
}
impl datom_codec::Datomic for Observation {
    fn incorporate(site: datom_codec::Site<'_>) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Locks" => std::result::Result::Ok(Self::Locks(datom_codec::Carrying::body(v)?)),
            _ => std::result::Result::Err(datom_codec::Headed::reject(
                &v,
                datom_codec::Problem::UnknownVariant(
                    protos::Word::try_from(v.name).expect("variant name"),
                ),
            )),
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for Observation {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(protos::Situated(
            protos::Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            match self {
                Self::Locks(p0) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("Locks").expect("static variant"),
                    std::boxed::Box::new(
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                    ),
                ),
            },
        ))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Lock(LockRequest),
    Release(LockId),
    Observe(ObserveSelection),
}
impl datom_codec::Datomic for Request {
    fn incorporate(site: datom_codec::Site<'_>) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Lock" => std::result::Result::Ok(Self::Lock(datom_codec::Carrying::body(v)?)),
            "Release" => std::result::Result::Ok(Self::Release(datom_codec::Carrying::body(v)?)),
            "Observe" => std::result::Result::Ok(Self::Observe(datom_codec::Carrying::body(v)?)),
            _ => std::result::Result::Err(datom_codec::Headed::reject(
                &v,
                datom_codec::Problem::UnknownVariant(
                    protos::Word::try_from(v.name).expect("variant name"),
                ),
            )),
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for Request {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(protos::Situated(
            protos::Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            match self {
                Self::Lock(p0) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("Lock").expect("static variant"),
                    std::boxed::Box::new(
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                    ),
                ),
                Self::Release(p0) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("Release").expect("static variant"),
                    std::boxed::Box::new(
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                    ),
                ),
                Self::Observe(p0) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("Observe").expect("static variant"),
                    std::boxed::Box::new(
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                    ),
                ),
            },
        ))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Response {
    Locked(Lock),
    Released(Lock),
    Observed(Observation),
    LockRejected(LockRejection),
    ReleaseRejected(ReleaseRejection),
}
impl datom_codec::Datomic for Response {
    fn incorporate(site: datom_codec::Site<'_>) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Locked" => std::result::Result::Ok(Self::Locked(datom_codec::Carrying::body(v)?)),
            "Released" => std::result::Result::Ok(Self::Released(datom_codec::Carrying::body(v)?)),
            "Observed" => std::result::Result::Ok(Self::Observed(datom_codec::Carrying::body(v)?)),
            "LockRejected" => {
                std::result::Result::Ok(Self::LockRejected(datom_codec::Carrying::body(v)?))
            }
            "ReleaseRejected" => {
                std::result::Result::Ok(Self::ReleaseRejected(datom_codec::Carrying::body(v)?))
            }
            _ => std::result::Result::Err(datom_codec::Headed::reject(
                &v,
                datom_codec::Problem::UnknownVariant(
                    protos::Word::try_from(v.name).expect("variant name"),
                ),
            )),
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for Response {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(protos::Situated(
            protos::Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            match self {
                Self::Locked(p0) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("Locked").expect("static variant"),
                    std::boxed::Box::new(
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                    ),
                ),
                Self::Released(p0) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("Released").expect("static variant"),
                    std::boxed::Box::new(
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                    ),
                ),
                Self::Observed(p0) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("Observed").expect("static variant"),
                    std::boxed::Box::new(
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                    ),
                ),
                Self::LockRejected(p0) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("LockRejected").expect("static variant"),
                    std::boxed::Box::new(
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                    ),
                ),
                Self::ReleaseRejected(p0) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("ReleaseRejected").expect("static variant"),
                    std::boxed::Box::new(
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                    ),
                ),
            },
        ))
    }
}
pub trait WireConversion: Sized {
    type Wire;
    fn into_wire(self) -> Self::Wire;
    fn try_from_wire(wire: Self::Wire) -> std::result::Result<Self, WireFault>;
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WireFault {
    Text,
}
pub type LockIdWire = i64;
pub type LockNameWire = std::string::String;
pub type FlowIdWire = std::string::String;
pub type LockPathWire = std::string::String;
pub type LockReasonWire = std::string::String;
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockRequestWire(
    pub LockNameWire,
    pub FlowIdWire,
    pub std::vec::Vec<LockPathWire>,
    pub LockReasonWire,
);
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockWire(
    pub LockIdWire,
    pub LockNameWire,
    pub FlowIdWire,
    pub std::vec::Vec<LockPathWire>,
    pub LockReasonWire,
);
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockOverlapWire(pub LockPathWire, pub LockWire);
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum LockRejectionWire {
    DuplicateName(LockWire),
    PathOverlap(LockOverlapWire),
}
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ReleaseRejectionWire {
    UnknownLockId,
}
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ObserveSelectionWire {
    Locks,
}
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ObservationWire {
    Locks(std::vec::Vec<LockWire>),
}
impl WireConversion for LockRequest {
    type Wire = LockRequestWire;
    fn into_wire(self) -> Self::Wire {
        let LockRequest(p0, p1, p2, p3) = self;
        LockRequestWire(
            p0.to_string(),
            p1.to_string(),
            p2.into_iter().map(|value| value.to_string()).collect(),
            p3.to_string(),
        )
    }
    fn try_from_wire(wire: Self::Wire) -> std::result::Result<Self, WireFault> {
        let LockRequestWire(p0, p1, p2, p3) = wire;
        Ok(LockRequest(
            protos::Text::try_from(p0).map_err(|_| WireFault::Text)?,
            protos::Text::try_from(p1).map_err(|_| WireFault::Text)?,
            p2.into_iter()
                .map(|value| protos::Text::try_from(value).map_err(|_| WireFault::Text))
                .collect::<std::result::Result<std::vec::Vec<_>, WireFault>>()?,
            protos::Text::try_from(p3).map_err(|_| WireFault::Text)?,
        ))
    }
}
impl WireConversion for Lock {
    type Wire = LockWire;
    fn into_wire(self) -> Self::Wire {
        let Lock(p0, p1, p2, p3, p4) = self;
        LockWire(
            p0,
            p1.to_string(),
            p2.to_string(),
            p3.into_iter().map(|value| value.to_string()).collect(),
            p4.to_string(),
        )
    }
    fn try_from_wire(wire: Self::Wire) -> std::result::Result<Self, WireFault> {
        let LockWire(p0, p1, p2, p3, p4) = wire;
        Ok(Lock(
            Ok(p0)?,
            protos::Text::try_from(p1).map_err(|_| WireFault::Text)?,
            protos::Text::try_from(p2).map_err(|_| WireFault::Text)?,
            p3.into_iter()
                .map(|value| protos::Text::try_from(value).map_err(|_| WireFault::Text))
                .collect::<std::result::Result<std::vec::Vec<_>, WireFault>>()?,
            protos::Text::try_from(p4).map_err(|_| WireFault::Text)?,
        ))
    }
}
impl WireConversion for LockOverlap {
    type Wire = LockOverlapWire;
    fn into_wire(self) -> Self::Wire {
        let LockOverlap(p0, p1) = self;
        LockOverlapWire(p0.to_string(), <Lock as WireConversion>::into_wire(p1))
    }
    fn try_from_wire(wire: Self::Wire) -> std::result::Result<Self, WireFault> {
        let LockOverlapWire(p0, p1) = wire;
        Ok(LockOverlap(
            protos::Text::try_from(p0).map_err(|_| WireFault::Text)?,
            <Lock as WireConversion>::try_from_wire(p1)?,
        ))
    }
}
impl WireConversion for LockRejection {
    type Wire = LockRejectionWire;
    fn into_wire(self) -> Self::Wire {
        match self {
            LockRejection::DuplicateName(value) => {
                LockRejectionWire::DuplicateName(<Lock as WireConversion>::into_wire(value))
            }
            LockRejection::PathOverlap(value) => {
                LockRejectionWire::PathOverlap(<LockOverlap as WireConversion>::into_wire(value))
            }
        }
    }
    fn try_from_wire(wire: Self::Wire) -> std::result::Result<Self, WireFault> {
        match wire {
            LockRejectionWire::DuplicateName(value) => Ok(LockRejection::DuplicateName(
                <Lock as WireConversion>::try_from_wire(value)?,
            )),
            LockRejectionWire::PathOverlap(value) => Ok(LockRejection::PathOverlap(
                <LockOverlap as WireConversion>::try_from_wire(value)?,
            )),
        }
    }
}
impl WireConversion for ReleaseRejection {
    type Wire = ReleaseRejectionWire;
    fn into_wire(self) -> Self::Wire {
        match self {
            ReleaseRejection::UnknownLockId => ReleaseRejectionWire::UnknownLockId,
        }
    }
    fn try_from_wire(wire: Self::Wire) -> std::result::Result<Self, WireFault> {
        match wire {
            ReleaseRejectionWire::UnknownLockId => Ok(ReleaseRejection::UnknownLockId),
        }
    }
}
impl WireConversion for ObserveSelection {
    type Wire = ObserveSelectionWire;
    fn into_wire(self) -> Self::Wire {
        match self {
            ObserveSelection::Locks => ObserveSelectionWire::Locks,
        }
    }
    fn try_from_wire(wire: Self::Wire) -> std::result::Result<Self, WireFault> {
        match wire {
            ObserveSelectionWire::Locks => Ok(ObserveSelection::Locks),
        }
    }
}
impl WireConversion for Observation {
    type Wire = ObservationWire;
    fn into_wire(self) -> Self::Wire {
        match self {
            Observation::Locks(value) => ObservationWire::Locks(
                value
                    .into_iter()
                    .map(|value| <Lock as WireConversion>::into_wire(value))
                    .collect(),
            ),
        }
    }
    fn try_from_wire(wire: Self::Wire) -> std::result::Result<Self, WireFault> {
        match wire {
            ObservationWire::Locks(value) => Ok(Observation::Locks(
                value
                    .into_iter()
                    .map(|value| <Lock as WireConversion>::try_from_wire(value))
                    .collect::<std::result::Result<std::vec::Vec<_>, WireFault>>()?,
            )),
        }
    }
}
impl WireConversion for Request {
    type Wire = RequestWire;
    fn into_wire(self) -> Self::Wire {
        match self {
            Request::Lock(value) => {
                RequestWire::Lock(<LockRequest as WireConversion>::into_wire(value))
            }
            Request::Release(value) => RequestWire::Release(value),
            Request::Observe(value) => {
                RequestWire::Observe(<ObserveSelection as WireConversion>::into_wire(value))
            }
        }
    }
    fn try_from_wire(wire: Self::Wire) -> std::result::Result<Self, WireFault> {
        match wire {
            RequestWire::Lock(value) => Ok(Request::Lock(
                <LockRequest as WireConversion>::try_from_wire(value)?,
            )),
            RequestWire::Release(value) => Ok(Request::Release(Ok(value)?)),
            RequestWire::Observe(value) => Ok(Request::Observe(
                <ObserveSelection as WireConversion>::try_from_wire(value)?,
            )),
        }
    }
}
impl WireConversion for Response {
    type Wire = ResponseWire;
    fn into_wire(self) -> Self::Wire {
        match self {
            Response::Locked(value) => {
                ResponseWire::Locked(<Lock as WireConversion>::into_wire(value))
            }
            Response::Released(value) => {
                ResponseWire::Released(<Lock as WireConversion>::into_wire(value))
            }
            Response::Observed(value) => {
                ResponseWire::Observed(<Observation as WireConversion>::into_wire(value))
            }
            Response::LockRejected(value) => {
                ResponseWire::LockRejected(<LockRejection as WireConversion>::into_wire(value))
            }
            Response::ReleaseRejected(value) => ResponseWire::ReleaseRejected(
                <ReleaseRejection as WireConversion>::into_wire(value),
            ),
        }
    }
    fn try_from_wire(wire: Self::Wire) -> std::result::Result<Self, WireFault> {
        match wire {
            ResponseWire::Locked(value) => Ok(Response::Locked(
                <Lock as WireConversion>::try_from_wire(value)?,
            )),
            ResponseWire::Released(value) => Ok(Response::Released(
                <Lock as WireConversion>::try_from_wire(value)?,
            )),
            ResponseWire::Observed(value) => Ok(Response::Observed(
                <Observation as WireConversion>::try_from_wire(value)?,
            )),
            ResponseWire::LockRejected(value) => Ok(Response::LockRejected(
                <LockRejection as WireConversion>::try_from_wire(value)?,
            )),
            ResponseWire::ReleaseRejected(value) => Ok(Response::ReleaseRejected(
                <ReleaseRejection as WireConversion>::try_from_wire(value)?,
            )),
        }
    }
}
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RequestWire {
    Lock(LockRequestWire),
    Release(LockIdWire),
    Observe(ObserveSelectionWire),
}
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ResponseWire {
    Locked(LockWire),
    Released(LockWire),
    Observed(ObservationWire),
    LockRejected(LockRejectionWire),
    ReleaseRejected(ReleaseRejectionWire),
}
