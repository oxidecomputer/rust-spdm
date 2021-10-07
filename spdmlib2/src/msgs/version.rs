use super::Msg;
use super::encoding::{Reader, ReadError, ReadErrorKind, WriteError, Writer};

pub struct GetVersion {}

impl Msg for GetVersion {
    fn name() -> &'static str {
        "GET_VERSION"
    }

    fn spdm_version() -> u8 {
        0x10
    }

    fn spdm_code() -> u8 {
        0x84
    }

    fn write_body(&self, w: &mut Writer) -> Result<usize, WriteError> {
        // Reserved bytes
        w.push(0)?;
        w.push(0)
    }
}

impl GetVersion {
    pub fn parse_body(buf: &[u8]) -> Result<GetVersion, ReadError> {
        if buf.len() < 2 {
            return Err(ReadError::new(Self::name(), ReadErrorKind::Empty));
        }
        // Reserved bytes
        if buf[0] != 0 || buf[1] != 0 {
            Err(ReadError::new(
                Self::name(),
                ReadErrorKind::ReservedByteNotZero,
            ))
        } else {
            Ok(GetVersion {})
        }
    }
}

pub const MAX_ALLOWED_VERSIONS: u8 = 2;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VersionEntry {
    pub major: u8,
    pub minor: u8,
    pub update: u8,
    pub alpha: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub num_entries: u8,
    pub entries: [VersionEntry; MAX_ALLOWED_VERSIONS as usize],
}

impl Version {
    fn empty() -> Version {
        Version {
            num_entries: 0,
            entries: [VersionEntry::default(); MAX_ALLOWED_VERSIONS as usize],
        }
    }
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
    fn name() -> &'static str {
        "VERSION"
    }

    fn spdm_version() -> u8 {
        0x10
    }

    fn spdm_code() -> u8 {
        0x04
    }

    fn write_body(&self, w: &mut Writer) -> Result<usize, WriteError> {
        // Reserved bytes
        w.push(0)?;
        w.push(0)?;
        w.push(0)?;

        w.push(self.num_entries)?;

        for v in self.entries.iter() {
            w.push(v.alpha | (v.update << 4))?;
            w.push(v.minor | (v.major << 4))?;
        }

        Ok(w.offset())
    }
}

impl Version {
    pub fn parse_body(buf: &[u8]) -> Result<Version, ReadError> {
        let mut reader = Reader::new(Self::name(), buf);

        // 3 reserved bytes
        for _ in 0..3 {
            let reserved = reader.read_byte()?;
            if reserved != 0 {
                return Err(ReadError::new(
                    Self::name(),
                    ReadErrorKind::ReservedByteNotZero,
                ));
            }
        }

        // 1 byte number of version entries
        let num_entries = reader.read_byte()?;
        if num_entries > MAX_ALLOWED_VERSIONS {
            return Err(ReadError::new(Self::name(), ReadErrorKind::TooManyEntries));
        }

        let mut version = Version::empty();
        version.num_entries = num_entries;

        // Num entries * 2 bytes
        for i in 0..(num_entries as usize) {
            version.entries[i] = VersionEntry {
                alpha: reader.read_bits(4)?,
                update: reader.read_bits(4)?,
                minor: reader.read_bits(4)?,
                major: reader.read_bits(4)?
            };
        }

        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_version_parses_correctly() {
        let mut buf =  [0u8; 16];
        let version = Version::default();
        assert_eq!(10, version.write(&mut buf).unwrap());
        assert_eq!(Ok(true), Version::parse_header(&buf));
        let version2 = Version::parse_body(&buf[2..]).unwrap();
        assert_eq!(version, version2);
    }
}
