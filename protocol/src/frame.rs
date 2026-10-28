use std::io::{self, Read, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;

pub const MAX_FRAME: usize = 64 * 1024 * 1024;

const HEADER: usize = 4;

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("frame of {length} bytes exceeds the {MAX_FRAME} byte limit")]
    TooLarge { length: usize },
    #[error("stream ended {missing} bytes before the end of a frame")]
    Truncated { missing: usize },
    #[error("frame is not a valid message: {0}")]
    Json(#[from] serde_json::Error),
    #[error("channel i/o failed: {0}")]
    Io(#[from] io::Error),
}

pub fn encode<M: Serialize>(message: &M) -> Result<Vec<u8>, FrameError> {
    let mut frame = vec![0; HEADER];
    serde_json::to_writer(&mut frame, message)?;
    let length = frame.len() - HEADER;
    if length > MAX_FRAME {
        return Err(FrameError::TooLarge { length });
    }
    frame[..HEADER].copy_from_slice(&(length as u32).to_le_bytes());
    Ok(frame)
}

pub fn write_frame<W: Write, M: Serialize>(writer: &mut W, message: &M) -> Result<(), FrameError> {
    writer.write_all(&encode(message)?)?;
    Ok(())
}

pub fn read_frame<R: Read, M: DeserializeOwned>(reader: &mut R) -> Result<Option<M>, FrameError> {
    let mut prefix = [0; HEADER];
    let received = fill(reader, &mut prefix)?;
    if received == 0 {
        return Ok(None);
    }
    if received < HEADER {
        return Err(FrameError::Truncated {
            missing: HEADER - received,
        });
    }
    let length = checked_length(prefix)?;
    let mut payload = vec![0; length];
    let received = fill(reader, &mut payload)?;
    if received < length {
        return Err(FrameError::Truncated {
            missing: length - received,
        });
    }
    Ok(Some(parse(&mut payload)?))
}

#[derive(Debug, Default)]
pub struct Decoder {
    buffer: Vec<u8>,
}

impl Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn decode<M: DeserializeOwned>(&mut self) -> Result<Option<M>, FrameError> {
        let Some(prefix) = self.buffer.first_chunk::<HEADER>() else {
            return Ok(None);
        };
        let length = checked_length(*prefix)?;
        let end = HEADER + length;
        if self.buffer.len() < end {
            return Ok(None);
        }
        let message = parse(&mut self.buffer[HEADER..end]);
        self.buffer.drain(..end);
        Ok(Some(message?))
    }

    pub fn buffered(&self) -> usize {
        self.buffer.len()
    }

    pub fn finish(self) -> Result<(), FrameError> {
        let missing = match self.buffer.first_chunk::<HEADER>() {
            None if self.buffer.is_empty() => return Ok(()),
            None => HEADER - self.buffer.len(),
            Some(prefix) => (HEADER + checked_length(*prefix)?).saturating_sub(self.buffer.len()),
        };
        Err(FrameError::Truncated { missing })
    }
}

fn checked_length(prefix: [u8; HEADER]) -> Result<usize, FrameError> {
    let length = u32::from_le_bytes(prefix) as usize;
    if length > MAX_FRAME {
        return Err(FrameError::TooLarge { length });
    }
    Ok(length)
}

fn parse<M: DeserializeOwned>(payload: &mut [u8]) -> Result<M, serde_json::Error> {
    replace_lone_surrogates(payload);
    serde_json::from_slice(payload)
}

const REPLACEMENT: &[u8; 6] = b"\\ufffd";

fn replace_lone_surrogates(json: &mut [u8]) {
    let mut index = 0;
    while index < json.len() {
        if json[index] != b'\\' {
            index += 1;
            continue;
        }
        let Some(unit) = escaped_unit(json, index) else {
            index += 2;
            continue;
        };
        let length = match unit {
            0xD800..=0xDBFF
                if escaped_unit(json, index + 6)
                    .is_some_and(|low| (0xDC00..=0xDFFF).contains(&low)) =>
            {
                12
            }
            0xD800..=0xDFFF => {
                json[index..index + 6].copy_from_slice(REPLACEMENT);
                6
            }
            _ => 6,
        };
        index += length;
    }
}

fn escaped_unit(json: &[u8], index: usize) -> Option<u16> {
    let escape = json.get(index..index + 6)?;
    if escape[..2] != *b"\\u" {
        return None;
    }
    let digits = std::str::from_utf8(&escape[2..]).ok()?;
    u16::from_str_radix(digits, 16).ok()
}

fn fill<R: Read>(reader: &mut R, buffer: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0;
    while filled < buffer.len() {
        match reader.read(&mut buffer[filled..]) {
            Ok(0) => break,
            Ok(count) => filled += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(filled)
}
