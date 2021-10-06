use codec::Writer;
use spdmlib::error::SpdmResult;
use spdmlib::spdm_err as err;

use super::Msg;

pub struct GetVersion {}

impl Msg for GetVersion {
    fn spdm_version() -> u8 {
        0x10
    }

    fn spdm_code() -> u8 {
        0x84
    }

    fn encode_body(&self, w: &mut Writer) -> Option<usize> {
        // Reserved bytes
        w.push(0)?;
        w.push(0)
    }
}

impl GetVersion {
    pub fn parse_body(buf: &[u8]) -> SpdmResult<GetVersion> {
        if buf.len() < 2 {
            return Err(err!(EINVAL));
        }
        if buf[0] != 0 || buf[1] != 0 {
            Err(err!(EINVAL))
        } else {
            Ok(GetVersion {})
        }
    }
}

const MAX_ALLOWED_VERSIONS: usize = 2;

pub struct VersionEntry {
    major: u8,
    minor: u8,
    update: u8,
    alpha: u8,
}

pub struct Version {
    num_entries: u8,

    // Just store versions encoded for simplicity.
    entries: [VersionEntry; MAX_ALLOWED_VERSIONS],
}

// There are only 2 published versions (1.0 and 1.1)
// They don't have update or alpha modifiers.
impl Default for Version {
    fn default() -> Version {
        Version {
            num_entries: 2,
            entries: [
                VersionEntry {
                    major: 1,
                    minor: 0,
                    update: 0,
                    alpha: 0,
                },
                VersionEntry {
                    major: 1,
                    minor: 1,
                    update: 0,
                    alpha: 0,
                },
            ],
        }
    }
}

impl Msg for Version {
    fn spdm_version() -> u8 {
        0x10
    }

    fn spdm_code() -> u8 {
        0x04
    }

    fn encode_body(&self, w: &mut Writer) -> Option<usize> {
        // Reserved bytes
        w.push(0)?;
        w.push(0)?;
        w.push(0)?;

        w.push(self.num_entries)?;

        for v in self.entries.iter() {
            w.push(v.alpha | (v.update << 4))?;
            w.push(v.minor | (v.major << 4))?;
        }

        Some(w.used())
    }
}

impl Version {}
