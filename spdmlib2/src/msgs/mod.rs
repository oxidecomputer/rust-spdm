use core::result::Result;

#[derive(Debug, Clone)]
pub struct WriteError {
    msg: &'static str,
    buf_size: usize,
}

impl WriteError {
    pub fn new(msg: &'static str, buf_size: usize) -> WriteError {
        WriteError { msg, buf_size }
    }
}


#[derive(Debug, Clone)]
pub enum ReadErrorKind {
    Header,
    NotEnoughInput,
    ReservedBytesNotZero
}

#[derive(Debug, Clone)]
pub struct ReadError {
    msg: &'static str,
    kind: ReadErrorKind
}

impl ReadError {
    pub fn new(msg: &'static str, kind: ReadErrorKind) -> ReadError {
        ReadError { msg, kind}
    }
}

pub struct Writer<'a> {
    msg: &'static str,
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> Writer<'a> {
    pub fn new(msg: &'static str, buf: &'a mut [u8]) -> Writer<'a> {
        Writer {
            msg,
            buf,
            offset: 0,
        }
    }

    /// Append a byte onto the buffer.
    ///
    /// Return the amount of the buffer used or an error if the buffer is full.
    pub fn push(&mut self, value: u8) -> Result<usize, WriteError> {
        if self.full() {
            Err(WriteError::new(self.msg, self.buf.len()))
        } else {
            self.buf[self.offset] = value;
            self.offset += 1;
            Ok(self.offset)
        }
    }

    pub fn full(&self) -> bool {
        self.offset == self.buf.len()
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

pub trait Msg {
    // Names should be written as in the spec (UPPER_SNAKE_CASE).
    fn name() -> &'static str;
    fn spdm_version() -> u8;
    fn spdm_code() -> u8;
    fn write_body(&self, w: &mut Writer) -> Result<usize, WriteError>;

    /// Parse the 2 byte message header and ensure the version field is
    /// correct for the given message type.
    ///
    /// Return `Ok(true)` if the writed header is of the given type and has a correct version.
    /// Return `Ok(false)` if the header is another message type.
    /// Return an error if the version is wrong for a GetVersion message.
    ///
    /// Prerequisite buf >= 2 bytes
    fn parse_header(buf: &[u8]) -> Result<bool, ReadError> {
        assert!(buf.len() > 2);
        if buf[1] != Self::spdm_code() {
            Ok(false)
        } else {
            if buf[0] == Self::spdm_version() {
                Ok(true)
            } else {
                Err(ReadError::new(Self::name(), ReadErrorKind::Header))
            }
        }
    }

    fn write(&self, buf: &mut [u8]) -> Result<usize, WriteError> {
        let mut w = Writer::new(Self::name(), buf);
        Self::write_header(&mut w)?;
        self.write_body(&mut w)
    }

    fn write_header(w: &mut Writer) -> Result<usize, WriteError> {
        w.push(Self::spdm_version())?;
        w.push(Self::spdm_code())
    }
}

pub mod version;
