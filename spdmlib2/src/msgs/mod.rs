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

#[derive(Debug, Clone)]
pub enum ReadErrorKind {
    Header,
    Empty,
    ReservedBytesNotZero,

    // An attempt to read one or more bytes not on a byte boundary
    Unaligned,

    // An attempt to read more than 7 bits in read_bits
    TooManyBits
}

#[derive(Debug, Clone)]
pub struct ReadError {
    msg: &'static str,
    kind: ReadErrorKind,
}

impl ReadError {
    pub fn new(msg: &'static str, kind: ReadErrorKind) -> ReadError {
        ReadError { msg, kind }
    }
}

pub struct Reader<'a> {
    msg: &'static str,
    buf: &'a [u8],
    byte_offset: usize,
    bit_offset: u8,
}

impl<'a> Reader<'a> {
    pub fn new(msg: &'static str, buf: &'a [u8]) -> Reader<'a> {
        Reader {
            msg,
            buf,
            byte_offset: 0,
            bit_offset: 0,
        }
    }

    pub fn read_byte(&mut self) -> Result<u8, ReadError> {
        if !self.aligned() {
            return Err(ReadError::new(self.msg, ReadErrorKind::Unaligned));
        }
        if self.empty() {
            return Err(ReadError::new(self.msg, ReadErrorKind::Empty));
        }
        let b = self.buf[self.byte_offset];
        self.byte_offset += 1;
        Ok(b)
    }

    // Allow reading up to 7 bits at a time.
    //
    // The read does not have to be aligned.
    pub fn read_bits(&mut self, count: u8) -> Result<u8, ReadError> {
       if count > 7 {
           return Err(ReadError::new(self.msg, ReadErrorKind::TooManyBits));
       }
       let mut new_bit_offset = self.bit_offset + count;
       if new_bit_offset >= 8 {
           let new_byte_offset = self.byte_offset + 1;
           new_bit_offset = new_bit_offset - 8;
           if new_byte_offset == self.buf.len() && new_bit_offset != 0 {
               Err(ReadError::new(self.msg, ReadErrorKind::Empty))
           } else {
               let mut b = self.buf[self.byte_offset] << self.bit_offset;
               let zero_bits = 8 - new_bit_offset;
               b |=  (self.buf[new_byte_offset] << zero_bits) >> zero_bits;
               self.byte_offset = new_byte_offset;
               self.bit_offset = new_bit_offset;
               Ok(b)
           }
       } else {
           let high  = self.bit_offset + count;
           let low = self.bit_offset;
           let b = (self.buf[self.byte_offset] << (8 - high)) >> (8 - high - low);
           self.bit_offset = high;
           Ok(b)
       }
       
    }

    pub fn empty(&self) -> bool {
        self.buf.len() == self.byte_offset
    }

    pub fn aligned(&self) -> bool {
        self.bit_offset == 0
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
