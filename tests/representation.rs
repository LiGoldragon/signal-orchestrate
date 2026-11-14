//! The scalar newtypes keep the representation the type aliases had: a value of
//! the contract archives to the same bytes, and writes the same Datom text, as a
//! mirror of it built from plain scalars.

use rkyv::rancor::Error;
use signal_orchestrate::{FlowId, HeldLock, LockId, LockName, LockPath, LockReason, LockRequest};

#[cfg_attr(
    feature = "datom",
    derive(datom_codec::Datomizable, datom_codec::Composing)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
struct PlainLock {
    lock_id: i64,
    lock_name: String,
    flow_id: String,
    lock_path_vector: Vec<String>,
    lock_reason: String,
}

#[cfg_attr(
    feature = "datom",
    derive(datom_codec::Datomizable, datom_codec::Composing)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
struct PlainRequest {
    lock_name: String,
    flow_id: String,
    lock_path_vector: Vec<String>,
    lock_reason: String,
}

fn lock() -> HeldLock {
    HeldLock {
        lock_id: LockId(7),
        lock_name: LockName("name".into()),
        flow_id: FlowId("flow".into()),
        lock_path_vector: vec![LockPath("/tmp/path".into())],
        lock_reason: LockReason("reason".into()),
    }
}

fn request() -> LockRequest {
    LockRequest {
        lock_name: LockName("name".into()),
        flow_id: FlowId("flow".into()),
        lock_path_vector: vec![LockPath("/tmp/path".into())],
        lock_reason: LockReason("reason".into()),
    }
}

fn plain_lock() -> PlainLock {
    PlainLock {
        lock_id: 7,
        lock_name: "name".into(),
        flow_id: "flow".into(),
        lock_path_vector: vec!["/tmp/path".into()],
        lock_reason: "reason".into(),
    }
}

fn plain_request() -> PlainRequest {
    PlainRequest {
        lock_name: "name".into(),
        flow_id: "flow".into(),
        lock_path_vector: vec!["/tmp/path".into()],
        lock_reason: "reason".into(),
    }
}

#[test]
fn newtype_values_archive_to_the_bytes_of_plain_scalars() {
    let new = rkyv::to_bytes::<Error>(&lock()).expect("archive lock");
    let old = rkyv::to_bytes::<Error>(&plain_lock()).expect("archive plain lock");
    assert_eq!(new.as_slice(), old.as_slice());
    let new = rkyv::to_bytes::<Error>(&request()).expect("archive request");
    let old = rkyv::to_bytes::<Error>(&plain_request()).expect("archive plain request");
    assert_eq!(new.as_slice(), old.as_slice());
}

#[cfg(feature = "datom")]
mod text {
    use super::*;
    use datom_codec::{Actualizing, Budget, Composing, Datomizable, Potential};
    use protos::{Protosizable, ReaderBudget, Textualizable};

    fn written<T: Datomizable>(value: &T) -> String {
        value.datomize(Vec::new()).protosize().textualize()
    }

    fn read<T: Composing>(text: &str) -> T {
        let mut budget = Budget {
            remaining: 4096,
            reader: ReaderBudget::try_from(4096usize).unwrap(),
            depth: 0,
            maximum_depth: 64,
        };
        Potential::<T>::from(text)
            .actualize(&mut budget)
            .expect("read the datom text")
    }

    #[test]
    fn newtype_values_write_the_text_of_plain_scalars() {
        assert_eq!(written(&lock()), written(&plain_lock()));
        assert_eq!(written(&request()), written(&plain_request()));
    }

    #[test]
    fn text_written_from_plain_scalars_reads_as_the_newtype_values() {
        assert_eq!(read::<HeldLock>(&written(&plain_lock())), lock());
        assert_eq!(read::<LockRequest>(&written(&plain_request())), request());
    }
}
