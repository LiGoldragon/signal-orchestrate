use crate::generated::signal::{
    CHANNEL_CONTRACT_ID, CHANNEL_WIRE_REVISION, ChannelContractId, ChannelWireRevision, Frame,
    PROTOCOL_VERSION, ProtocolVersion,
};

/// The hand-owned binary envelope boundary for this generated contract.
pub trait SignalFrameCodec: Sized {
    fn encode_length_prefixed(&self) -> Result<Vec<u8>, FrameCodecError>;
    fn decode_length_prefixed(bytes: &[u8]) -> Result<Self, FrameCodecError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameCodecError {
    LengthPrefixMissing,
    LengthMismatch {
        expected: usize,
        found: usize,
    },
    LengthTooLarge,
    ArchiveEncode,
    ArchiveDecode,
    WrongChannelContract {
        expected: ChannelContractId,
        found: ChannelContractId,
    },
    WrongChannelWireRevision {
        expected: ChannelWireRevision,
        found: ChannelWireRevision,
    },
    UnsupportedProtocol {
        expected: ProtocolVersion,
        found: ProtocolVersion,
    },
}

impl SignalFrameCodec for Frame {
    fn encode_length_prefixed(&self) -> Result<Vec<u8>, FrameCodecError> {
        validate(self)?;
        let archive = rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .map_err(|_| FrameCodecError::ArchiveEncode)?;
        let length = u32::try_from(archive.len()).map_err(|_| FrameCodecError::LengthTooLarge)?;
        let mut frame = Vec::with_capacity(4 + archive.len());
        frame.extend_from_slice(&length.to_le_bytes());
        frame.extend_from_slice(&archive);
        Ok(frame)
    }

    fn decode_length_prefixed(bytes: &[u8]) -> Result<Self, FrameCodecError> {
        let Some(prefix) = bytes.get(..4) else {
            return Err(FrameCodecError::LengthPrefixMissing);
        };
        let expected = u32::from_le_bytes([prefix[0], prefix[1], prefix[2], prefix[3]]) as usize;
        let payload = &bytes[4..];
        if payload.len() != expected {
            return Err(FrameCodecError::LengthMismatch {
                expected,
                found: payload.len(),
            });
        }
        let frame = rkyv::from_bytes::<Self, rkyv::rancor::Error>(payload)
            .map_err(|_| FrameCodecError::ArchiveDecode)?;
        validate(&frame)?;
        Ok(frame)
    }
}

fn validate(frame: &Frame) -> Result<(), FrameCodecError> {
    if frame.channel_contract_id != CHANNEL_CONTRACT_ID {
        return Err(FrameCodecError::WrongChannelContract {
            expected: CHANNEL_CONTRACT_ID,
            found: frame.channel_contract_id,
        });
    }
    if frame.channel_wire_revision != CHANNEL_WIRE_REVISION {
        return Err(FrameCodecError::WrongChannelWireRevision {
            expected: CHANNEL_WIRE_REVISION,
            found: frame.channel_wire_revision,
        });
    }
    if frame.protocol_version != PROTOCOL_VERSION {
        return Err(FrameCodecError::UnsupportedProtocol {
            expected: PROTOCOL_VERSION,
            found: frame.protocol_version,
        });
    }
    Ok(())
}
