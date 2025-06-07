use rusqlite::{Connection, Result, params};
use std::path::Path;

use crate::types::Embedding;

pub struct VectorStore {
    conn: Connection,
}

impl VectorStore {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS embeddings (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                chunk_start_line INTEGER NOT NULL,
                chunk_end_line INTEGER NOT NULL,
                vector BLOB NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn insert_many_embeddings_bulk(&mut self, embeddings: &[Embedding]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO embeddings 
            (id, path, chunk_index, chunk_start_line, chunk_end_line, vector)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;

            for emb in embeddings {
                let bytes = f32_slice_to_bytes(&emb.vector);
                stmt.execute(params![
                    emb.id,
                    emb.path,
                    emb.chunk_index,
                    emb.chunk_start_line,
                    emb.chunk_end_line,
                    bytes
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn insert_embedding(&self, emb: &Embedding) -> Result<()> {
        let bytes = f32_slice_to_bytes(&emb.vector);
        self.conn.execute(
            "INSERT OR REPLACE INTO embeddings
            (id, path, chunk_index, chunk_start_line, chunk_end_line, vector)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                emb.id,
                emb.path,
                emb.chunk_index as u32,
                emb.chunk_start_line as u32,
                emb.chunk_end_line as u32,
                bytes
            ],
        )?;
        Ok(())
    }

    pub fn list_embeddings(&self) -> Result<Vec<Embedding>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, chunk_index, chunk_start_line, chunk_end_line, vector FROM embeddings"
        )?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let path: String = row.get(1)?;
            let chunk_index: u32 = row.get(2)?;
            let start: u32 = row.get(3)?;
            let end: u32 = row.get(4)?;
            let blob: Vec<u8> = row.get(5)?;
            let vector = bytes_to_f32_vec(&blob)?;
            Ok(Embedding {
                id,
                path,
                chunk_index: chunk_index as usize,
                chunk_start_line: start as usize,
                chunk_end_line: end as usize,
                vector,
            })
        })?;
        rows.collect()
    }
}

// Conversion utils identiques
fn f32_slice_to_bytes(vec: &[f32]) -> Vec<u8> {
    let len = vec.len();
    let ptr = vec.as_ptr() as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, len * 4).to_vec() }
}

fn bytes_to_f32_vec(bytes: &[u8]) -> Result<Vec<f32>> {
    if bytes.len() % 4 != 0 {
        return Err(rusqlite::Error::FromSqlConversionFailure(
            bytes.len(),
            rusqlite::types::Type::Blob,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Blob size not a multiple of 4",
            )),
        ));
    }
    let len = bytes.len() / 4;
    let ptr = bytes.as_ptr() as *const f32;
    unsafe { Ok(std::slice::from_raw_parts(ptr, len).to_vec()) }
}
