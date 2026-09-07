use hmac::{Hmac, Mac};
use sha2::Sha512;

use crate::{ByteSource, Error};

type HmacSha512 = Hmac<Sha512>;

pub(crate) struct HmacByteStream<'a> {
    key: &'a [u8],
    context: &'a [u8],
    counter: u64,
    buffer: [u8; 64],
    offset: usize,
}

impl<'a> HmacByteStream<'a> {
    pub(crate) fn new(key: &'a [u8], context: &'a [u8]) -> Result<Self, Error> {
        let mut stream = Self {
            key,
            context,
            counter: 0,
            buffer: [0; 64],
            offset: 64,
        };
        stream.refill()?;
        Ok(stream)
    }

    fn refill(&mut self) -> Result<(), Error> {
        let mut mac = HmacSha512::new_from_slice(self.key).map_err(|_| Error::HmacKey)?;
        mac.update(self.context);
        mac.update(&self.counter.to_be_bytes());
        self.buffer.copy_from_slice(&mac.finalize().into_bytes());
        self.counter += 1;
        self.offset = 0;
        Ok(())
    }
}

impl ByteSource for HmacByteStream<'_> {
    fn next_byte(&mut self) -> Result<u8, Error> {
        if self.offset >= self.buffer.len() {
            self.refill()?;
        }
        let b = self.buffer[self.offset];
        self.offset += 1;
        Ok(b)
    }
}
