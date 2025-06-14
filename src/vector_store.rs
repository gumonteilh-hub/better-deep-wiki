use crate::types::{Chunk, Embedding};
use qdrant_client::Qdrant;
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder, UpsertPointsBuilder,
    VectorParamsBuilder,
};
use std::collections::HashMap;
use std::iter::Iterator;

pub struct VectorStore {
    client: Qdrant,
    collection_name: String,
    vector_dim: usize,
}

impl VectorStore {
    pub async fn reset_or_create(collection_name: &str, vector_dim: usize) -> Result<Self, String> {
        let config = QdrantConfig::from_url("http://qdrant:6334");
        let client = Qdrant::new(config).map_err(|e| e.to_string())?;

        let exists = client
            .collection_exists(collection_name)
            .await
            .map_err(|e| e.to_string())?;
        if exists {
            client
                .delete_collection(collection_name)
                .await
                .expect("Error trying to reset collection");
        }
        client
            .create_collection(
                CreateCollectionBuilder::new(collection_name).vectors_config(
                    VectorParamsBuilder::new(vector_dim as u64, Distance::Cosine),
                ),
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(Self {
            client,
            collection_name: collection_name.to_string(),
            vector_dim,
        })
    }

    pub async fn try_open(collection_name: &str, vector_dim: usize) -> Result<Self, String> {
        let config = QdrantConfig::from_url("http://qdrant:6334");
        let client = Qdrant::new(config).map_err(|e| e.to_string())?;

        let exists = client
            .collection_exists(collection_name)
            .await
            .map_err(|e| e.to_string())?;
        if !exists {
            return Err(format!("No collection {} found", collection_name));
        }

        Ok(Self {
            client,
            collection_name: collection_name.to_string(),
            vector_dim,
        })
    }

    pub async fn insert_many_embeddings_bulk(
        &self,
        embeddings: &[Embedding],
    ) -> Result<(), String> {
        let points: Vec<PointStruct> = embeddings
            .iter()
            .filter(|e| {
                let ok = e.vector.len() == self.vector_dim && !e.vector.is_empty();
                if !ok {
                    eprintln!("SKIP embedding id={} len={}", e.id, e.vector.len());
                }
                ok
            })
            .map(|emb| {
                let mut payload = HashMap::new();
                payload.insert("path".to_string(), emb.chunk.path.clone().into());
                payload.insert(
                    "chunk_index".to_string(),
                    (emb.chunk.chunk_index as i64).into(),
                );
                payload.insert(
                    "chunk_start_line".to_string(),
                    (emb.chunk.chunk_start_line as i64).into(),
                );
                payload.insert(
                    "chunk_end_line".to_string(),
                    (emb.chunk.chunk_end_line as i64).into(),
                );
                payload.insert(
                    "chunk_text".to_string(),
                    (emb.chunk.text.clone() as String).into(),
                );

                PointStruct::new(emb.id.clone(), emb.vector.clone(), payload)
            })
            .collect();

        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.collection_name, points))
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn search_top_k(
        &self,
        query_vector: &[f32],
        top_k: u64,
    ) -> Result<Vec<Chunk>, String> {
        let search_builder =
            SearchPointsBuilder::new(&self.collection_name, query_vector.to_vec(), top_k)
                .with_payload(true);

        let resp = self
            .client
            .search_points(search_builder)
            .await
            .map_err(|e| e.to_string())?;

        let results = resp
            .result
            .iter()
            .map(|pt| Self::extract_payload(&pt).unwrap())
            .collect();

        Ok(results)
    }

    fn extract_payload(pt: &qdrant_client::qdrant::ScoredPoint) -> Result<Chunk, &'static str> {
        let payload = &pt.payload;

        let path = payload
            .get("path")
            .and_then(|v| v.kind.as_ref())
            .and_then(|kind| match kind {
                qdrant_client::qdrant::value::Kind::StringValue(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or("Missing or invalid 'path' in payload")?;

        let chunk_index = payload
            .get("chunk_index")
            .and_then(|v| v.kind.as_ref())
            .and_then(|kind| match kind {
                qdrant_client::qdrant::value::Kind::IntegerValue(i) => Some(*i as usize),
                _ => None,
            })
            .ok_or("Missing or invalid 'chunk_index' in payload")?;

        let chunk_start_line = payload
            .get("chunk_start_line")
            .and_then(|v| v.kind.as_ref())
            .and_then(|kind| match kind {
                qdrant_client::qdrant::value::Kind::IntegerValue(i) => Some(*i as usize),
                _ => None,
            })
            .ok_or("Missing or invalid 'chunk_start_line' in payload")?;

        let chunk_end_line = payload
            .get("chunk_end_line")
            .and_then(|v| v.kind.as_ref())
            .and_then(|kind| match kind {
                qdrant_client::qdrant::value::Kind::IntegerValue(i) => Some(*i as usize),
                _ => None,
            })
            .ok_or("Missing or invalid 'chunk_end_line' in payload")?;

        let chunk_text = payload
            .get("chunk_text")
            .and_then(|v| v.kind.as_ref())
            .and_then(|kind| match kind {
                qdrant_client::qdrant::value::Kind::StringValue(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or("Missing or invalid 'chunk_end_line' in payload")?;

        Ok(Chunk {
            path,
            chunk_index,
            chunk_start_line,
            chunk_end_line,
            text: chunk_text,
        })
    }
}
