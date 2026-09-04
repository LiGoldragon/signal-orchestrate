use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

// ---------------------------------------------------------------------------
// Helper: prepend a path index to a fault (replicates datomic's private Prepending)
// ---------------------------------------------------------------------------

fn prepend_fault(fault: datomic::Fault, index: i64) -> datomic::Fault {
    match fault {
        datomic::Fault::Structural(f) => datomic::Fault::Structural(f),
        datomic::Fault::Conceptual(mut path, problem) => {
            path.insert(0, index);
            datomic::Fault::Conceptual(path, problem)
        }
        datomic::Fault::Corporal(mut path, problem) => {
            path.insert(0, index);
            datomic::Fault::Corporal(path, problem)
        }
    }
}

// ---------------------------------------------------------------------------
// Wire envelope types (rkyv only, no Datomic)
// ---------------------------------------------------------------------------

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Version(pub u16, pub u16, pub u16);

pub const SIGNAL_VERSION: Version = Version(1u16, 0u16, 0u16);

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    VersionMismatch(Version, Version),
    Unreadable,
}

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

pub type LockId = i64;

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockRequest(
    pub protos::Text,
    pub protos::Text,
    pub Vec<protos::Text>,
    pub protos::Text,
);

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct Lock(
    pub i64,
    pub protos::Text,
    pub protos::Text,
    pub Vec<protos::Text>,
    pub protos::Text,
);

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct LockOverlap(pub protos::Text, pub Lock);

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
pub enum Observation {
    Locks(Vec<Lock>),
}

// ---------------------------------------------------------------------------
// Request / Reply / Body / Frame
// ---------------------------------------------------------------------------

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Lock(LockRequest),
    Release(LockId),
    Observe(ObserveSelection),
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Locked(Lock),
    Released(Lock),
    Observed(Observation),
    LockRejected(LockRejection),
    ReleaseRejected(ReleaseRejection),
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum Body {
    Request(Request),
    Reply(Reply),
    Refusal(Refusal),
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct Frame(pub Version, pub Body);

// ===========================================================================
// Datomic impls for domain types
// ===========================================================================

// ---------------------------------------------------------------------------
// LockRequest: struct (Text, Text, Vec<Text>, Text)
// ---------------------------------------------------------------------------

impl protos::Corporal<datomic::Datom> for LockRequest {
    type Fault = datomic::Fault;
    fn incorporate(datom: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        let datomic::Datom::Struct(fields) = datom else {
            return Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Struct, datom),
            ));
        };
        if fields.len() != 4 {
            return Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Arity(4, fields.len() as i64),
            ));
        }
        let mut iter = fields.into_iter();
        Ok(Self(
            <protos::Text as protos::Corporal<datomic::Datom>>::incorporate(iter.next().unwrap())
                .map_err(|f| prepend_fault(f, 0))?,
            <protos::Text as protos::Corporal<datomic::Datom>>::incorporate(iter.next().unwrap())
                .map_err(|f| prepend_fault(f, 1))?,
            <Vec<protos::Text> as protos::Corporal<datomic::Datom>>::incorporate(iter.next().unwrap())
                .map_err(|f| prepend_fault(f, 2))?,
            <protos::Text as protos::Corporal<datomic::Datom>>::incorporate(iter.next().unwrap())
                .map_err(|f| prepend_fault(f, 3))?,
        ))
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

// ---------------------------------------------------------------------------
// Lock: struct (i64, Text, Text, Vec<Text>, Text)
// ---------------------------------------------------------------------------

impl protos::Corporal<datomic::Datom> for Lock {
    type Fault = datomic::Fault;
    fn incorporate(datom: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        let datomic::Datom::Struct(fields) = datom else {
            return Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Struct, datom),
            ));
        };
        if fields.len() != 5 {
            return Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Arity(5, fields.len() as i64),
            ));
        }
        let mut iter = fields.into_iter();
        Ok(Self(
            <i64 as protos::Corporal<datomic::Datom>>::incorporate(iter.next().unwrap())
                .map_err(|f| prepend_fault(f, 0))?,
            <protos::Text as protos::Corporal<datomic::Datom>>::incorporate(iter.next().unwrap())
                .map_err(|f| prepend_fault(f, 1))?,
            <protos::Text as protos::Corporal<datomic::Datom>>::incorporate(iter.next().unwrap())
                .map_err(|f| prepend_fault(f, 2))?,
            <Vec<protos::Text> as protos::Corporal<datomic::Datom>>::incorporate(iter.next().unwrap())
                .map_err(|f| prepend_fault(f, 3))?,
            <protos::Text as protos::Corporal<datomic::Datom>>::incorporate(iter.next().unwrap())
                .map_err(|f| prepend_fault(f, 4))?,
        ))
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

// ---------------------------------------------------------------------------
// LockOverlap: struct (Text, Lock)
// ---------------------------------------------------------------------------

impl protos::Corporal<datomic::Datom> for LockOverlap {
    type Fault = datomic::Fault;
    fn incorporate(datom: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        let datomic::Datom::Struct(fields) = datom else {
            return Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Struct, datom),
            ));
        };
        if fields.len() != 2 {
            return Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Arity(2, fields.len() as i64),
            ));
        }
        let mut iter = fields.into_iter();
        Ok(Self(
            <protos::Text as protos::Corporal<datomic::Datom>>::incorporate(iter.next().unwrap())
                .map_err(|f| prepend_fault(f, 0))?,
            <Lock as protos::Corporal<datomic::Datom>>::incorporate(iter.next().unwrap())
                .map_err(|f| prepend_fault(f, 1))?,
        ))
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

// ---------------------------------------------------------------------------
// LockRejection: enum { DuplicateName(Lock), PathOverlap(LockOverlap) }
// ---------------------------------------------------------------------------

impl protos::Corporal<datomic::Datom> for LockRejection {
    type Fault = datomic::Fault;
    fn incorporate(datom: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        let datomic::Datom::Variant(ref head, ref sep, ref body) = datom else {
            return Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, datom),
            ));
        };
        if *sep != protos::Separator::Period {
            return Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Separator(*sep),
            ));
        }
        match head.as_str() {
            "DuplicateName" => {
                let inner = body
                    .as_ref()
                    .map(|b| *b.clone())
                    .ok_or_else(|| {
                        datomic::Fault::Corporal(
                            vec![],
                            datomic::Problem::Shape(
                                datomic::Expected::Struct,
                                datomic::Datom::Bare(head.clone()),
                            ),
                        )
                    })?;
                Ok(Self::DuplicateName(
                    <Lock as protos::Corporal<datomic::Datom>>::incorporate(inner)?,
                ))
            }
            "PathOverlap" => {
                let inner = body
                    .as_ref()
                    .map(|b| *b.clone())
                    .ok_or_else(|| {
                        datomic::Fault::Corporal(
                            vec![],
                            datomic::Problem::Shape(
                                datomic::Expected::Struct,
                                datomic::Datom::Bare(head.clone()),
                            ),
                        )
                    })?;
                Ok(Self::PathOverlap(
                    <LockOverlap as protos::Corporal<datomic::Datom>>::incorporate(inner)?,
                ))
            }
            _ => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::UnknownVariant(head.clone()),
            )),
        }
    }
}

impl datomic::Datomic for LockRejection {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::DuplicateName(v) => datomic::Datom::Variant(
                "DuplicateName".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
            Self::PathOverlap(v) => datomic::Datom::Variant(
                "PathOverlap".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// ReleaseRejection: unit-variant enum { UnknownLockId }
// ---------------------------------------------------------------------------

impl protos::Corporal<datomic::Datom> for ReleaseRejection {
    type Fault = datomic::Fault;
    fn incorporate(datom: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match &datom {
            datomic::Datom::Bare(s) if s == "UnknownLockId" => Ok(Self::UnknownLockId),
            _ => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, datom),
            )),
        }
    }
}

impl datomic::Datomic for ReleaseRejection {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::UnknownLockId => datomic::Datom::Bare("UnknownLockId".to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// ObserveSelection: unit-variant enum { Locks }
// ---------------------------------------------------------------------------

impl protos::Corporal<datomic::Datom> for ObserveSelection {
    type Fault = datomic::Fault;
    fn incorporate(datom: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match &datom {
            datomic::Datom::Bare(s) if s == "Locks" => Ok(Self::Locks),
            _ => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, datom),
            )),
        }
    }
}

impl datomic::Datomic for ObserveSelection {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::Locks => datomic::Datom::Bare("Locks".to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Observation: enum { Locks(Vec<Lock>) }
// ---------------------------------------------------------------------------

impl protos::Corporal<datomic::Datom> for Observation {
    type Fault = datomic::Fault;
    fn incorporate(datom: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        let datomic::Datom::Variant(ref head, ref sep, ref body) = datom else {
            return Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, datom),
            ));
        };
        if *sep != protos::Separator::Period {
            return Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Separator(*sep),
            ));
        }
        match head.as_str() {
            "Locks" => {
                let inner = body
                    .as_ref()
                    .map(|b| *b.clone())
                    .ok_or_else(|| {
                        datomic::Fault::Corporal(
                            vec![],
                            datomic::Problem::Shape(
                                datomic::Expected::Vector,
                                datomic::Datom::Bare(head.clone()),
                            ),
                        )
                    })?;
                Ok(Self::Locks(
                    <Vec<Lock> as protos::Corporal<datomic::Datom>>::incorporate(inner)?,
                ))
            }
            _ => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::UnknownVariant(head.clone()),
            )),
        }
    }
}

impl datomic::Datomic for Observation {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::Locks(v) => datomic::Datom::Variant(
                "Locks".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Request: mixed enum { Lock(LockRequest), Release(LockId), Observe(ObserveSelection) }
// ---------------------------------------------------------------------------

impl protos::Corporal<datomic::Datom> for Request {
    type Fault = datomic::Fault;
    fn incorporate(datom: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match &datom {
            datomic::Datom::Variant(head, sep, body) => {
                if *sep != protos::Separator::Period {
                    return Err(datomic::Fault::Corporal(
                        vec![],
                        datomic::Problem::Separator(*sep),
                    ));
                }
                match head.as_str() {
                    "Lock" => {
                        let inner = body
                            .as_ref()
                            .map(|b| *b.clone())
                            .ok_or_else(|| {
                                datomic::Fault::Corporal(
                                    vec![],
                                    datomic::Problem::Shape(
                                        datomic::Expected::Struct,
                                        datomic::Datom::Bare(head.clone()),
                                    ),
                                )
                            })?;
                        Ok(Self::Lock(
                            <LockRequest as protos::Corporal<datomic::Datom>>::incorporate(inner)?,
                        ))
                    }
                    "Release" => {
                        let inner = body
                            .as_ref()
                            .map(|b| *b.clone())
                            .ok_or_else(|| {
                                datomic::Fault::Corporal(
                                    vec![],
                                    datomic::Problem::Shape(
                                        datomic::Expected::Bare,
                                        datomic::Datom::Bare(head.clone()),
                                    ),
                                )
                            })?;
                        Ok(Self::Release(
                            <i64 as protos::Corporal<datomic::Datom>>::incorporate(inner)?,
                        ))
                    }
                    "Observe" => {
                        let inner = body
                            .as_ref()
                            .map(|b| *b.clone())
                            .ok_or_else(|| {
                                datomic::Fault::Corporal(
                                    vec![],
                                    datomic::Problem::Shape(
                                        datomic::Expected::Variant,
                                        datomic::Datom::Bare(head.clone()),
                                    ),
                                )
                            })?;
                        Ok(Self::Observe(
                            <ObserveSelection as protos::Corporal<datomic::Datom>>::incorporate(inner)?,
                        ))
                    }
                    _ => Err(datomic::Fault::Corporal(
                        vec![],
                        datomic::Problem::UnknownVariant(head.clone()),
                    )),
                }
            }
            _ => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, datom),
            )),
        }
    }
}

impl datomic::Datomic for Request {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::Lock(v) => datomic::Datom::Variant(
                "Lock".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
            Self::Release(v) => datomic::Datom::Variant(
                "Release".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
            Self::Observe(v) => datomic::Datom::Variant(
                "Observe".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Reply: mixed enum
// ---------------------------------------------------------------------------

impl protos::Corporal<datomic::Datom> for Reply {
    type Fault = datomic::Fault;
    fn incorporate(datom: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match &datom {
            datomic::Datom::Variant(head, sep, body) => {
                if *sep != protos::Separator::Period {
                    return Err(datomic::Fault::Corporal(
                        vec![],
                        datomic::Problem::Separator(*sep),
                    ));
                }
                match head.as_str() {
                    "Locked" => {
                        let inner = body
                            .as_ref()
                            .map(|b| *b.clone())
                            .ok_or_else(|| {
                                datomic::Fault::Corporal(
                                    vec![],
                                    datomic::Problem::Shape(
                                        datomic::Expected::Struct,
                                        datomic::Datom::Bare(head.clone()),
                                    ),
                                )
                            })?;
                        Ok(Self::Locked(
                            <Lock as protos::Corporal<datomic::Datom>>::incorporate(inner)?,
                        ))
                    }
                    "Released" => {
                        let inner = body
                            .as_ref()
                            .map(|b| *b.clone())
                            .ok_or_else(|| {
                                datomic::Fault::Corporal(
                                    vec![],
                                    datomic::Problem::Shape(
                                        datomic::Expected::Struct,
                                        datomic::Datom::Bare(head.clone()),
                                    ),
                                )
                            })?;
                        Ok(Self::Released(
                            <Lock as protos::Corporal<datomic::Datom>>::incorporate(inner)?,
                        ))
                    }
                    "Observed" => {
                        let inner = body
                            .as_ref()
                            .map(|b| *b.clone())
                            .ok_or_else(|| {
                                datomic::Fault::Corporal(
                                    vec![],
                                    datomic::Problem::Shape(
                                        datomic::Expected::Variant,
                                        datomic::Datom::Bare(head.clone()),
                                    ),
                                )
                            })?;
                        Ok(Self::Observed(
                            <Observation as protos::Corporal<datomic::Datom>>::incorporate(inner)?,
                        ))
                    }
                    "LockRejected" => {
                        let inner = body
                            .as_ref()
                            .map(|b| *b.clone())
                            .ok_or_else(|| {
                                datomic::Fault::Corporal(
                                    vec![],
                                    datomic::Problem::Shape(
                                        datomic::Expected::Variant,
                                        datomic::Datom::Bare(head.clone()),
                                    ),
                                )
                            })?;
                        Ok(Self::LockRejected(
                            <LockRejection as protos::Corporal<datomic::Datom>>::incorporate(inner)?,
                        ))
                    }
                    "ReleaseRejected" => {
                        let inner = body
                            .as_ref()
                            .map(|b| *b.clone())
                            .ok_or_else(|| {
                                datomic::Fault::Corporal(
                                    vec![],
                                    datomic::Problem::Shape(
                                        datomic::Expected::Variant,
                                        datomic::Datom::Bare(head.clone()),
                                    ),
                                )
                            })?;
                        Ok(Self::ReleaseRejected(
                            <ReleaseRejection as protos::Corporal<datomic::Datom>>::incorporate(inner)?,
                        ))
                    }
                    _ => Err(datomic::Fault::Corporal(
                        vec![],
                        datomic::Problem::UnknownVariant(head.clone()),
                    )),
                }
            }
            _ => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, datom),
            )),
        }
    }
}

impl datomic::Datomic for Reply {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::Locked(v) => datomic::Datom::Variant(
                "Locked".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
            Self::Released(v) => datomic::Datom::Variant(
                "Released".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
            Self::Observed(v) => datomic::Datom::Variant(
                "Observed".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
            Self::LockRejected(v) => datomic::Datom::Variant(
                "LockRejected".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
            Self::ReleaseRejected(v) => datomic::Datom::Variant(
                "ReleaseRejected".to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(v))),
            ),
        }
    }
}
