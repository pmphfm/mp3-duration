use std::io;
use std::io::prelude::*;
use std::time::Duration;

use crate::error::*;

pub struct Context<'r, T> {
    reader: &'r mut T,
    bytes_read: usize,
    reached_eof: bool,
    pub duration: Duration,
}

impl<'r, T: Read + Seek> Context<'r, T> {
    pub fn new(reader: &'r mut T) -> Self {
        Context {
            reader: reader,
            bytes_read: 0,
            duration: Duration::from_secs(0),
            reached_eof: false,
        }
    }

    pub fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<(), MP3DurationError> {
        let result = self.reader.read_exact(buffer);
        if result.is_ok() {
            self.bytes_read += buffer.len();
        }
        self.reached_eof = match &result {
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => true,
            _ => false,
        };
        result.map_err(|e| self.error(e.into()))
    }

    pub fn skip(&mut self, num_bytes: usize) -> Result<(), MP3DurationError> {
        let cur_pos = self
            .reader
            .seek(std::io::SeekFrom::Current(0))
            .map_err(|e| self.error(e.into()))?;
        let end_pos = self
            .reader
            .seek(std::io::SeekFrom::End(0))
            .map_err(|e| self.error(e.into()))?;

        if (cur_pos as usize + num_bytes) > end_pos as usize {
            return Err(self.error(ErrorKind::UnexpectedEOF));
        }

        self.reader
            .seek(std::io::SeekFrom::Start(cur_pos + num_bytes as u64))
            .map_err(|e| self.error(e.into()))
            .map(|_| ())
    }

    pub fn reached_eof(&self) -> bool {
        self.reached_eof
    }

    pub fn error(&self, e: ErrorKind) -> MP3DurationError {
        MP3DurationError {
            kind: e,
            offset: self.bytes_read,
            at_duration: self.duration,
        }
    }
}
