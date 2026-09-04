use crate::generated::signal::{Frame, SIGNAL_VERSION, Version};

pub trait SignalFrameCodec: Sized {
    fn encode_length_prefixed(&self) -> Result<Vec<u8>, FrameCodecError>;
    fn decode_length_prefixed(bytes: &[u8]) -> Result<Self, FrameCodecError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameCodecError {
    LengthPrefixMissing,
    LengthMismatch { expected: usize, found: usize },
    LengthTooLarge,
    ArchiveEncode,
    ArchiveDecode,
    VersionMismatch { expected: Version, found: Version },
}

impl SignalFrameCodec for Frame {
    fn encode_length_prefixed(&self) -> Result<Vec<u8>, FrameCodecError> {
        if self.0 != SIGNAL_VERSION {
            return Err(FrameCodecError::VersionMismatch {
                expected: SIGNAL_VERSION,
                found: self.0,
            });
        }
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
        if frame.0 != SIGNAL_VERSION {
            return Err(FrameCodecError::VersionMismatch {
                expected: SIGNAL_VERSION,
                found: frame.0,
            });
        }
        Ok(frame)
    }
}
