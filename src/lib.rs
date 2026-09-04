//! Generated ordinary Orchestrate Signal contract and hand-owned frame codec.

pub mod codec;
pub mod generated;

pub use codec::*;
pub use generated::signal::*;

/// The authored ethos source for this signal contract.
pub const ETHOS_SOURCE: &str = include_str!("../ethos/signal.ethos");
