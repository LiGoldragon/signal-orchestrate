pub mod generated;
pub use generated::signal::*;

pub const ETHOS: &str = include_str!("../ethos/signal.ethos");

pub fn archive_query(value: &Query) -> Vec<u8> {
    rkyv::to_bytes::<rkyv::rancor::Error>(value)
        .expect("archive query")
        .to_vec()
}

pub fn restore_query(bytes: &[u8]) -> Result<Query, rkyv::rancor::Error> {
    rkyv::from_bytes(bytes)
}

pub fn archive_response(value: &Response) -> Vec<u8> {
    rkyv::to_bytes::<rkyv::rancor::Error>(value)
        .expect("archive response")
        .to_vec()
}

pub fn restore_response(bytes: &[u8]) -> Result<Response, rkyv::rancor::Error> {
    rkyv::from_bytes(bytes)
}
