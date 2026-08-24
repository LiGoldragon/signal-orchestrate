use dotos::{DotosEncode, DotosSource};
use signal_frame::{
    ClientFrame, ExchangeIdentifier, ExchangeLane, LaneSequence, RequestPayload, SessionEpoch,
    WireContract,
};
use signal_orchestrate::{
    Frame, OrchestrateRequest, OrchestrateWire, PathLock, PathLockDescription, PathLockName,
    PathLockPath, PathLockPaths, PathLockRegistered, PathLockRegistrationRefusal,
    PathLockRegistrationRejected, PathLockRelease, PathLockReleaseRefusal, PathLockReleaseRejected,
    PathLockReleased,
};

#[test]
fn generated_contract_textualizes_register_and_release() {
    assert_eq!(OrchestrateWire::BINDING.contract().value(), 1);
    assert_eq!(OrchestrateWire::BINDING.revision().value(), 4);
    let path_lock = PathLock {
        path_lock_name: PathLockName("orchestrate-interfaces".into()),
        path_lock_paths: PathLockPaths(vec![PathLockPath(
            "/git/github.com/LiGoldragon/signal-orchestrate".into(),
        )]),
        path_lock_description: PathLockDescription("generated contract witness".into()),
    };
    let register = OrchestrateRequest::Register(path_lock.clone());
    let release_payload = PathLockRelease {
        path_lock_name: PathLockName("orchestrate-interfaces".into()),
    };
    let release = OrchestrateRequest::Release(release_payload.clone());
    let registered = PathLockRegistered {
        path_lock: path_lock.clone(),
    };
    let registration_rejected = PathLockRegistrationRejected {
        path_lock: path_lock.clone(),
        path_lock_registration_refusal: PathLockRegistrationRefusal::DuplicateActiveName(
            path_lock.clone(),
        ),
    };
    let released = PathLockReleased {
        path_lock_release: release_payload.clone(),
    };
    let release_rejected = PathLockReleaseRejected {
        path_lock_release: release_payload.clone(),
        path_lock_release_refusal: PathLockReleaseRefusal::UnknownActiveName,
    };

    assert_eq!(
        path_lock.to_dotos(),
        "PathLock.{orchestrate-interfaces [/git/github.com/LiGoldragon/signal-orchestrate] (generated contract witness)}"
    );
    assert_eq!(
        release_payload.to_dotos(),
        "PathLockRelease.{orchestrate-interfaces}"
    );
    assert_eq!(
        registered.to_dotos(),
        "PathLockRegistered.{orchestrate-interfaces [/git/github.com/LiGoldragon/signal-orchestrate] (generated contract witness)}"
    );
    assert_eq!(
        released.to_dotos(),
        "PathLockReleased.{orchestrate-interfaces}"
    );
    assert_eq!(
        registration_rejected.to_dotos(),
        "PathLockRegistrationRejected.{{orchestrate-interfaces [/git/github.com/LiGoldragon/signal-orchestrate] (generated contract witness)} DuplicateActiveName.{orchestrate-interfaces [/git/github.com/LiGoldragon/signal-orchestrate] (generated contract witness)}}"
    );
    assert_eq!(
        release_rejected.to_dotos(),
        "PathLockReleaseRejected.{{orchestrate-interfaces} UnknownActiveName}"
    );
    assert_eq!(
        DotosSource::new(&path_lock.to_dotos())
            .parse::<PathLock>()
            .expect("decode path lock"),
        path_lock
    );
    assert_eq!(
        DotosSource::new(&release_payload.to_dotos())
            .parse::<PathLockRelease>()
            .expect("decode release"),
        release_payload
    );
    assert_eq!(
        DotosSource::new(&registration_rejected.to_dotos())
            .parse::<PathLockRegistrationRejected>()
            .expect("decode registration rejection"),
        registration_rejected
    );
    assert_eq!(
        DotosSource::new(&release_rejected.to_dotos())
            .parse::<PathLockReleaseRejected>()
            .expect("decode release rejection"),
        release_rejected
    );
    let exchange = ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    );
    let frame = Frame::request_frame(exchange, register.into_request()).expect("frame register");
    let bytes = frame.encode_client_frame().expect("encode register frame");
    assert_eq!(
        Frame::decode_client_frame(&bytes).expect("decode register frame"),
        frame
    );
    assert!(matches!(release, OrchestrateRequest::Release(_)));
}
