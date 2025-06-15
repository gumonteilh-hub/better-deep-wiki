use crate::types::{Chunk, Embedding};
use qdrant_client::Qdrant;
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder, UpsertPointsBuilder,
    VectorParamsBuilder, PayloadSchemaType, CreateFieldIndexCollectionBuilder, FieldType,
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
            
            // Wait for the collection to be deleted
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        client
            .create_collection(
                CreateCollectionBuilder::new(collection_name).vectors_config(
                    VectorParamsBuilder::new(vector_dim as u64, Distance::Cosine),
                ),
            )
            .await
            .map_err(|e| e.to_string())?;

        // Créer un index sur le champ chunk_text pour accélérer la recherche textuelle
        client
            .create_field_index(
                CreateFieldIndexCollectionBuilder::new(
                    collection_name, 
                    "chunk_text", 
                    FieldType::Text
                ).field_type(PayloadSchemaType::Text),
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
                    (emb.chunk.chunk_index.to_string()).into(),
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
                payload.insert(
                    "function_name".to_string(),
                    emb.chunk.function_name.clone().unwrap_or("".to_string()).into(),
                );
                payload.insert(
                    "chunk_type".to_string(),
                    format!("{:?}", emb.chunk.chunk_type).into(),
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

    pub async fn hybrid_search(
        &self,
        query_vector: &[f32],
        query_text: &str,
        top_k: u64,
    ) -> Result<Vec<Chunk>, String> {
        let (semantic_results, lexical_results) = tokio::join!(
            self.search_top_k(query_vector, top_k * 2),
            self.lexical_search(query_text, top_k * 2)
        );
        
        let semantic_results = semantic_results?;
        let lexical_results = lexical_results?;
        
        let fused_results = self.reciprocal_rank_fusion(semantic_results, lexical_results, top_k);
        
        Ok(fused_results)
    }

    async fn lexical_search(&self, query_text: &str, top_k: u64) -> Result<Vec<Chunk>, String> {
        use qdrant_client::qdrant::{Filter, Condition, FieldCondition, Match};
        
        let query_lower = query_text.to_lowercase();
        let query_terms: Vec<&str> = query_lower
            .split_whitespace()
            .filter(|term| term.len() > 2)
            .collect();

        if query_terms.is_empty() {
            return Ok(vec![]);
        }

        let mut conditions = Vec::new();
        
        for term in query_terms {
            conditions.push(Condition {
                condition_one_of: Some(
                    qdrant_client::qdrant::condition::ConditionOneOf::Field(
                        FieldCondition {
                            key: "chunk_text".to_string(),
                            r#match: Some(Match {
                                match_value: Some(
                                    qdrant_client::qdrant::r#match::MatchValue::Text(term.to_string())
                                ),
                            }),
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            geo_polygon: None,
                            datetime_range: None,
                            values_count: None,
                            is_empty: None,
                            is_null: None,
                        }
                    )
                ),
            });
        }

        let filter = Filter {
            should: conditions,
            must: vec![],
            must_not: vec![],
            min_should: None,
        };

        // Recherche avec filtre textuel - on utilise un vecteur dummy pour la recherche
        let dummy_vector = vec![0.0; self.vector_dim];
        let search_builder = SearchPointsBuilder::new(&self.collection_name, dummy_vector, top_k)
            .with_payload(true)
            .filter(filter);

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

    fn reciprocal_rank_fusion(
        &self,
        semantic_results: Vec<Chunk>,
        lexical_results: Vec<Chunk>,
        top_k: u64,
    ) -> Vec<Chunk> {
        use std::collections::HashMap;

        let k = std::env::var("HYBRID_SEARCH_RRF_K")
            .unwrap_or_else(|_| "60.0".to_string())
            .parse::<f32>()
            .unwrap_or(60.0);

        let mut chunk_scores: HashMap<String, f32> = HashMap::new();
        let mut all_chunks: HashMap<String, Chunk> = HashMap::new();

        for (rank, chunk) in semantic_results.iter().enumerate() {
            let chunk_id = format!("{}:{}", chunk.path, chunk.chunk_index);
            let rrf_score = 1.0 / (k + (rank + 1) as f32);
            
            *chunk_scores.entry(chunk_id.clone()).or_insert(0.0) += rrf_score;
            all_chunks.insert(chunk_id, chunk.clone());
        }

        for (rank, chunk) in lexical_results.iter().enumerate() {
            let chunk_id = format!("{}:{}", chunk.path, chunk.chunk_index);
            let rrf_score = 1.0 / (k + (rank + 1) as f32);
            
            *chunk_scores.entry(chunk_id.clone()).or_insert(0.0) += rrf_score;
            all_chunks.insert(chunk_id, chunk.clone());
        }

        let mut scored_chunks: Vec<_> = chunk_scores
            .into_iter()
            .filter_map(|(chunk_id, score)| {
                all_chunks.get(&chunk_id).map(|chunk| (chunk.clone(), score))
            })
            .collect();

        scored_chunks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored_chunks
            .into_iter()
            .take(top_k as usize)
            .map(|(chunk, _score)| chunk)
            .collect()
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
                qdrant_client::qdrant::value::Kind::StringValue(s) => Some(s.clone()),
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
            .ok_or("Missing or invalid 'chunk_text' in payload")?;

        let function_name = payload
            .get("function_name")
            .and_then(|v| v.kind.as_ref())
            .and_then(|kind| match kind {
                qdrant_client::qdrant::value::Kind::StringValue(s) => {
                    if s.is_empty() { None } else { Some(s.clone()) }
                },
                _ => None,
            });

        let chunk_type = payload
            .get("chunk_type")
            .and_then(|v| v.kind.as_ref())
            .and_then(|kind| match kind {
                qdrant_client::qdrant::value::Kind::StringValue(s) => {
                    match s.as_str() {
                        "Function" => Some(crate::types::ChunkType::Function),
                        "Class" => Some(crate::types::ChunkType::Class),
                        "Method" => Some(crate::types::ChunkType::Method),
                        "Interface" => Some(crate::types::ChunkType::Interface),
                        "Struct" => Some(crate::types::ChunkType::Struct),
                        "Impl" => Some(crate::types::ChunkType::Impl),
                        "LineChunk" | _ => Some(crate::types::ChunkType::LineChunk),
                    }
                },
                _ => Some(crate::types::ChunkType::LineChunk),
            })
            .unwrap_or(crate::types::ChunkType::LineChunk);

        Ok(Chunk {
            path,
            chunk_index: chunk_index.to_string(),
            chunk_start_line,
            chunk_end_line,
            text: chunk_text,
            function_name,
            chunk_type,
        })
    }
}
