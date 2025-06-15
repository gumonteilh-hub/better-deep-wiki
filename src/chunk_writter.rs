use crate::types::Chunk;
use bincode::Decode;
use bincode::error::EncodeError;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, ErrorKind, Write};
use std::sync::{Arc, Mutex};

pub struct ChunkBinWriter {
    inner: Arc<Mutex<BufWriter<File>>>,
}

impl ChunkBinWriter {
    pub fn create(path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(BufWriter::new(file))),
        })
    }

    pub fn write(&self, chunk: &Chunk) -> Result<(), EncodeError> {
        let mut writer = self.inner.lock().unwrap();
        bincode::encode_into_std_write::<Chunk, _, _>(
            chunk.clone(),
            &mut *writer,
            bincode::config::standard(),
        )
        .map(|_| ())
    }
    pub fn flush(&self) -> std::io::Result<()> {
        let mut writer = self.inner.lock().unwrap();
        writer.flush()
    }
}

unsafe impl Send for ChunkBinWriter {}
unsafe impl Sync for ChunkBinWriter {}

use bincode::{config::standard, decode_from_std_read, error::DecodeError};
use std::io::BufReader;

pub struct ChunkBinReader<T> {
    reader: BufReader<File>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ChunkBinReader<T> {
    pub fn open(path: &str) -> std::io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::new(file),
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<T: Decode<()>> Iterator for ChunkBinReader<T> {
    type Item = Result<T, DecodeError>;
    fn next(&mut self) -> Option<Self::Item> {
        let config = standard();
        match decode_from_std_read::<T, _, _>(&mut self.reader, config) {
            Ok(val) => Some(Ok(val)),
            Err(DecodeError::UnexpectedEnd { .. }) => None,
            Err(DecodeError::Io { inner, .. }) if inner.kind() == ErrorKind::UnexpectedEof => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_minimal_bincode_v2() {
        let file_path = "test_simple.bin";
        let _ = std::fs::remove_file(file_path);
        let writer = ChunkBinWriter::create(file_path).unwrap();
        let chunk = Chunk {
            path: "a".into(),
            chunk_index: 1.to_string(),
            chunk_end_line: 5,
            chunk_start_line: 0,
            text: "yo".into(),
            function_name: None,
            chunk_type: crate::types::ChunkType::Class,
        };
        writer.write(&chunk).unwrap();
        writer.flush().unwrap();
        let mut reader = ChunkBinReader::<Chunk>::open(file_path).unwrap();
        let chunk2 = reader.next().unwrap().unwrap();
        assert_eq!(chunk2.text, "yo");
        let _ = fs::remove_file(file_path);
    }
}
