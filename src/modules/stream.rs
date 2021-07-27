use std::io::{Bytes, Error, ErrorKind, Read, Result, Write};

pub struct StreamReader<T> {
    bytes: Bytes<T>,   // All bytes read
    next: u128,        // next byte
    buffer_len: usize, // size of each byte
}

impl<T: Read> StreamReader<T> {
    pub fn new(bytes: T) -> Self {
        Self {
            bytes: bytes.bytes(),
            next: 0,
            buffer_len: 0,
        }
    }

    pub fn shift(&mut self, n: usize) -> Result<u64> {
        while self.buffer_len < n {
            let x = match self.bytes.next().transpose()? {
                Some(x) => x as u128,
                None => {
                    return Err(Error::new(
                        ErrorKind::UnexpectedEof,
                        "Unexpected end of bitstream :(",
                    ))
                }
            };

            self.next = (self.next << 8) | x;
            self.buffer_len += 8;
        }

        Ok(((self.next >> (self.buffer_len - n)) & (0xffff_ffff_ffff_ffff >> (64 - n))) as u64)
    }

    pub fn read(&mut self, n: usize) -> Result<u64> {
        let b = self.shift(n)?;
        self.buffer_len -= n;
        Ok(b)
    }
}

pub struct StreamWriter<T: Write> {
    bytes: T,          // All bytes read
    next: u128,        // next byte
    buffer_len: usize, // size of each byte
}

impl<T: Write> StreamWriter<T> {
    pub fn new(bytes: T) -> Self {
        Self {
            bytes,
            next: 0,
            buffer_len: 0,
        }
    }

    pub fn write(&mut self, bits: u64, mut len: usize) -> Result<()> {
        while len >= 128 {
            self.write(0, 64)?;
            len -= 64;
        }

        if len > 64 {
            self.write(0, len - 64)?;
            len = 64;
        }

        self.next = (self.next << len) | bits as u128;
        self.buffer_len += len;

        while self.buffer_len >= 8 {
            let next = (self.next >> (self.buffer_len - 8)) as u8;
            self.bytes.write_all(&[next])?;
            self.buffer_len -= 8;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        if self.buffer_len > 0 {
            let next_byte = (self.next << (8 - self.buffer_len)) as u8;
            self.bytes.write_all(&[next_byte])?;
            self.buffer_len = 0;
        }
        self.bytes.flush()
    }
}

impl<T: Write> Drop for StreamWriter<T> {
    fn drop(&mut self) {
        self.flush().expect("Error flushing to disk")
    }
}
