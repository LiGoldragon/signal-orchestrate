pub mod generated;
pub use generated::signal::*;

pub const ETHOS: &str = include_str!("../ethos/signal.ethos");

/// The portable binary representation shared by Signal peers.
pub trait SignalArchive: Sized {
    fn archive(&self) -> Vec<u8>;
    fn restore(bytes: &[u8]) -> Result<Self, rkyv::rancor::Error>;
}

impl SignalArchive for Query {
    fn archive(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .expect("archive query")
            .to_vec()
    }

    fn restore(bytes: &[u8]) -> Result<Self, rkyv::rancor::Error> {
        rkyv::from_bytes(bytes)
    }
}

impl SignalArchive for Response {
    fn archive(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .expect("archive response")
            .to_vec()
    }

    fn restore(bytes: &[u8]) -> Result<Self, rkyv::rancor::Error> {
        rkyv::from_bytes(bytes)
    }
}
