use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chunk {
    pub path: String,
    pub index: usize,
    pub text: String,
}

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
    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }
    pub fn write(&self, chunk: &Chunk) -> bincode::Result<()> {
        let mut writer = self.inner.lock().unwrap();
        bincode::serialize_into(&mut *writer, chunk)
    }
    pub fn flush(&self) -> std::io::Result<()> {
        let mut writer = self.inner.lock().unwrap();
        writer.flush()
    }
}

unsafe impl Send for ChunkBinWriter {}
unsafe impl Sync for ChunkBinWriter {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;

    #[test]
    fn test_threadsafe_chunkbin() {
        let file_path = "test_thread_chunks.bin";
        let _ = fs::remove_file(file_path);
        let writer = ChunkBinWriter::create(file_path).unwrap().arc();
        let handles: Vec<_> = (0..4)
            .map(|t| {
                let writer = Arc::clone(&writer);
                thread::spawn(move || {
                    for i in 0..10 {
                        let chunk = Chunk {
                            path: format!("/th{}_{}.rs", t, i),
                            index: i,
                            text: format!("x = {}", i),
                        };
                        writer.write(&chunk).unwrap();
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        writer.flush().unwrap();
        // Relire pour vérifier
        let reader = std::fs::File::open(file_path).unwrap();
        let count = bincode::de::Deserializer::from_reader(reader)
            .into_iter::<Chunk>()
            .count();
        assert_eq!(count, 40);
    }
}
