use codec:: Writer;
use spdmlib::error::SpdmResult;
use spdmlib::spdm_err as err;

pub trait Msg {
    fn spdm_version() -> u8;
    fn spdm_code() -> u8;
    fn encode_body(&self, w: &mut Writer) -> Option<usize>;

    /// Parse the 2 byte message header and ensure the version field is
    /// correct for the given message type.
    ///
    /// Return `Ok(true)` if the encoded header is of the given type and has a correct version.
    /// Return `Ok(false)` if the header is another message type.
    /// Return an error if the version is wrong for a GetVersion message.
    ///
    /// Prerequisite buf >= 2 bytes
    fn parse_header(buf: &[u8]) -> SpdmResult<bool> {
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

    fn encode(&self, buf: &mut [u8]) -> SpdmResult<usize> {
        let mut w = Writer::init(buf);
        Self::encode_header(&mut w).ok_or_else(|| err!(ENOMEM))?;
        self.encode_body(&mut w).ok_or_else(|| err!(ENOMEM))
    }

    fn encode_header(w: &mut Writer) -> Option<usize> {
        w.push(Self::spdm_version())?;
        w.push(Self::spdm_code())
    }
}

pub mod version;
