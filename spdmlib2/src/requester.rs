//! A requester follows the typestate pattern
//! https://cliffle.com/blog/rust-typestate/
//!
//!
//! As this code is no_std, we can't use a box to minimize the size of the type
//! states. Therefore we limit the contained state, and pass in any large state
//! when needed by given parameters. We pass in parameters rather than store
//! mutable references, because we also want States to be Send, so we can use
//! them in async code outside a no_std environment.

use crate::msgs::version::MAX_ALLOWED_VERSIONS;
use crate::msgs::{GetVersion, Msg, ReadError, Version, VersionEntry, WriteError};

const HEADER_SIZE: usize = 2;
const TRANSCRIPT_SIZE: usize = 1024;

/// A `Transcript` is used to track contigous operations for measurement
/// purposes.
///
/// A Transcript spans multiple states, and is purposefully kept outside those
/// states to reduce the cost of the typestate pattern which takes and returns
/// states by value.
pub struct Transcript {
    buf: [u8; TRANSCRIPT_SIZE],
    offset: usize,
}

impl Transcript {
    pub fn new() -> Transcript {
        Transcript {
            buf: [0; TRANSCRIPT_SIZE],
            offset: 0,
        }
    }

    pub fn extend(&mut self, buf: &[u8]) -> Result<(), WriteError> {
        let end = self.offset + buf.len();
        if end > self.buf.len() {
            Err(WriteError::new("TRANSCRIPT", buf.len()))
        } else {
            self.buf[self.offset..end].copy_from_slice(buf);
            self.offset = end;
            Ok(())
        }
    }

    pub fn clear(&mut self) {
        self.offset = 0;
    }

    pub fn get(&self) -> &[u8] {
        &self.buf[0..self.offset]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequesterError {
    Write(WriteError),
    Read(ReadError),

    // `got` is the code. TODO: Try to map this to a message name?
    UnexpectedMsg { expected: &'static str, got: u8 },

    //
    // Version related messages
    //
    TooManyVersions { received: u8 },
    NoSupportedVersions { received: Version },
}

/// The possible sef of state transitions out of the VersionState.
pub enum VersionTransition {
    Capabilities(CapabilitiesState),
}

/// A Requester starts in this state, where version negotiation is attempted.
pub struct VersionState {}

impl VersionState {
    pub fn write_get_version(&mut self, buf: &mut [u8]) -> Result<usize, WriteError> {
        GetVersion {}.write(buf)
    }

    // Only Version messages are acceptable here.
    pub fn handle_msg(
        self,
        buf: &[u8],
        transcript: &mut Transcript,
    ) -> Result<VersionTransition, RequesterError> {
        match Version::parse_header(buf) {
            Ok(true) => self.handle_version(buf, transcript),
            Ok(false) => Err(RequesterError::UnexpectedMsg {
                expected: Version::name(),
                got: buf[0],
            }),
            Err(e) => Err(RequesterError::Read(e)),
        }
    }

    fn handle_version(
        self,
        buf: &[u8],
        transcript: &mut Transcript,
    ) -> Result<VersionTransition, RequesterError> {
        let version =
            Version::parse_body(&buf[HEADER_SIZE..]).map_err(|e| RequesterError::Read(e))?;
        if version.num_entries > MAX_ALLOWED_VERSIONS {
            return Err(RequesterError::TooManyVersions {
                received: version.num_entries,
            });
        }
        if let Some(version_entry) = Self::find_max_matching_version(&version) {
            // SUCCESS!
            Self::transcribe(buf, transcript)?;
            let new_state = CapabilitiesState {
                version: version_entry,
            };
            Ok(VersionTransition::Capabilities(new_state))
        } else {
            Err(RequesterError::NoSupportedVersions { received: version })
        }
    }

    fn find_max_matching_version(version: &Version) -> Option<VersionEntry> {
        let expected = Version::default();
        let mut found = VersionEntry::default();

        for i in 0..version.num_entries as usize {
            if version.entries[i] > found {
                for j in 0..expected.num_entries as usize {
                    if version.entries[i] == expected.entries[j] {
                        found = version.entries[i];
                        break;
                    }
                }
            }
        }

        if found == VersionEntry::default() {
            None
        } else {
            Some(found)
        }
    }

    fn transcribe(response: &[u8], transcript: &mut Transcript) -> Result<(), RequesterError> {
        // We always send the same request.
        let mut request = [0u8; 4];
        let size = GetVersion {}.write(&mut request).unwrap();
        assert_eq!(size, 4);
        transcript.extend(&request).map_err(|e| RequesterError::Write(e))?;
        transcript.extend(response).map_err(|e| RequesterError::Write(e))
    }
}

// After version negotiation, capabilities are negotiated.
pub struct CapabilitiesState {
    version: VersionEntry,
}
