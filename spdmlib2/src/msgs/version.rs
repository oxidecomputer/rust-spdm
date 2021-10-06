use codec::Writer;
use spdmlib::error::SpdmResult;
use spdmlib::spdm_err as err;

pub struct GetVersion {}

impl GetVersion {
    pub fn spdm_version() -> u8 {
        0x10
    }

    pub fn spdm_code() -> u8 {
        0x84
    }

    pub fn encode(&self, buf: &mut [u8]) -> SpdmResult<usize> {
        let mut w = Writer::init(buf);
        self.write(&mut w).ok_or_else(||err!(ENOMEM))
    }

    /// Parse the 2 byte message header.
    /// Return `Ok(true)` if the encoded header is a correct GetVersion header.
    /// Return `Ok(false)` if the header is another message type.
    /// Return an error if the version is wrong for a GetVersion message.
    ///
    /// Prerequisite buf >= 2 bytes
    pub fn parse_header(buf: &[u8]) -> SpdmResult<bool> {
        assert!(buf.len() > 2);
        if buf[1] != Self::spdm_code() {
            Ok(false)
        } else {
            if buf[0] == Self::spdm_version() {
                Ok(true)
            } else {
                Err(err!(EINVAL))
            }
        }
    }

    pub fn parse_body(&self, buf: &[u8]) -> SpdmResult<GetVersion> {
        if buf.len() < 2 {
            return Err(err!(EINVAL));
        }
        if buf[0] != 0 || buf[1] != 0 {
            Err(err!(EINVAL))
        } else {
            Ok(GetVersion{})
        }
    }

    fn write(&self, w: &mut Writer) -> Option<usize> {
        w.push(Self::spdm_version())?;
        w.push(Self::spdm_code())?;

        // Reserved bytes
        w.push(0)?;
        w.push(0)
    }
}

pub struct Version {}

impl Version {
    pub fn spdm_version() -> u8 {
        0x10
    }

    pub fn spdm_code() -> u8 {
        0x04
    }

    pub fn encode(&self, buf: &mut [u8]) -> SpdmResult<usize> {
        let mut w = Writer::init(buf);
        self.write(&mut w).ok_or_else(||err!(ENOMEM))
    }

    fn write(&self, w: &mut Writer) -> Option<usize> {
        w.push(Self::spdm_version())?;
        w.push(Self::spdm_code())?;

        // Reserved bytes
        w.push(0)?;
        w.push(0)?;
        w.push(0)?;

        // Number of versions supported.
        w.push(2)?;

        // There are only 2 published versions (1.1 and 1.2)
        // They don't have update or alpha modifiers
        w.push(0)?;
        w.push(0x10)?;
        w.push(0x11)
    }
}
