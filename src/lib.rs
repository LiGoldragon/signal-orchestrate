pub mod generated;
pub use generated::signal::*;

use std::marker::PhantomData;

pub const ETHOS: &str = include_str!("../ethos/signal.ethos");

/// A portable rkyv Signal frame whose target contract is carried in its type.
pub struct SignalFrame<T> {
    bytes: Vec<u8>,
    target: PhantomData<fn() -> T>,
}

/// Data that can form a portable Signal frame.
pub trait SignalFraming: Sized {
    fn frame(&self) -> SignalFrame<Self>;
}

/// A frame exposes its peer-wire bytes for transport framing.
pub trait ByteViewing {
    fn bytes(&self) -> &[u8];
}

/// A typed portable frame can restore the contract value it carries.
pub trait Restoring {
    type Restored;

    fn restore(&self) -> Result<Self::Restored, rkyv::rancor::Error>;
}

impl SignalFraming for Query {
    fn frame(&self) -> SignalFrame<Self> {
        SignalFrame {
            bytes: rkyv::to_bytes::<rkyv::rancor::Error>(self)
                .expect("archive query")
                .to_vec(),
            target: PhantomData,
        }
    }
}

impl SignalFraming for Response {
    fn frame(&self) -> SignalFrame<Self> {
        SignalFrame {
            bytes: rkyv::to_bytes::<rkyv::rancor::Error>(self)
                .expect("archive response")
                .to_vec(),
            target: PhantomData,
        }
    }
}

impl<T> ByteViewing for SignalFrame<T> {
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Restoring for SignalFrame<Query> {
    type Restored = Query;

    fn restore(&self) -> Result<Self::Restored, rkyv::rancor::Error> {
        rkyv::from_bytes(self.bytes())
    }
}

impl Restoring for SignalFrame<Response> {
    type Restored = Response;

    fn restore(&self) -> Result<Self::Restored, rkyv::rancor::Error> {
        rkyv::from_bytes(self.bytes())
    }
}
