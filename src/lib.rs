use std::{fs, path::Path};

use crate::{embedding::Embedder, vector_store::VectorStore};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

mod chatter;
mod chunk_writter;
mod chunking;
mod embedding;
mod parsing;
mod types;
mod utils;
mod vector_store;

use tokio::runtime::Builder;
use types::Chunk;

pub fn scan_repo(repo_path: String) {
    let repo_identifier = utils::compute_repo_identifier(&repo_path);
    if !fs::exists("generated/".to_string() + &repo_identifier).unwrap() {
        println!("Start parsing repo");
        let meta_files = parsing::parse_repo(repo_path);
        println!("{} files detected.\nStart chunking", meta_files.len());

        let splitter = chunking::TextSplitter {
            split_by: chunking::SplitBy::Line,
            chunk_size: 350,
            chunk_overlap: 100,
        };

        let writter = chunk_writter::ChunkBinWriter::create(
            format!("generated/{}", &repo_identifier).as_str(),
        )
        .unwrap();
        meta_files.par_iter().for_each(|meta| {
            let path = Path::new(&meta.path);
            match splitter.split_file(path) {
                Ok(chunks) => {
                    for chunk in chunks {
                        writter.write(&chunk).unwrap();
                    }
                }
                Err(e) => eprintln!("Erreur sur {:?} : {}", meta.path, e),
            }
        });
        writter.flush().unwrap();
        println!("Finish preparing chunks");
    }

    let reader = chunk_writter::ChunkBinReader::<Chunk>::open(
        format!("generated/{}", &repo_identifier).as_str(),
    )
    .unwrap();
    let all_chunks: Vec<_> = reader
        .map(|r| match r {
            Ok(chunk) => {
                println!("{}: {}", chunk.path, chunk.chunk_index);
                chunk
            }
            Err(e) => {
                panic!("Error deconding chunks : {e}")
            }
        })
        .collect();

    utils::calculate_cost(&all_chunks);

    println!("Start embedding");

    let embedder = embedding::MistralEmbedder::from_env();
    let batches = utils::make_batches(all_chunks);

    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    rt.block_on(async {
        let db = VectorStore::reset_or_create(&repo_identifier, 1536)
            .await
            .expect("Erreur init vectorstore");

        for batch in batches {
            match embedder.embed_batch(batch).await {
                Ok(embs) => match db.insert_many_embeddings_bulk(&embs).await {
                    Ok(_) => (),
                    Err(_) => eprintln!("Error saving vectors in db"),
                },
                Err(e) => {
                    panic!("{e}");
                }
            }
        }
    })
}

pub fn ask_repo(question: String, instructions: String, repo_path: String) -> Result<String, String> {
    let repo_identifier = utils::compute_repo_identifier(&repo_path);

    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");
    rt.block_on(async {
        let db = VectorStore::try_open(&repo_identifier, 1536).await?;

        let embedder = embedding::MistralEmbedder::from_env();
        let q_vec = embedder.embed_question(question.clone()).await?;

        let top_k = 10;
        let similar_chunks = db
            .search_top_k(&q_vec, top_k)
            .await
            .map_err(|e| format!("Vector search failed: {e}"))?;

        // 4. Construction du prompt contextuel pour le LLM (concatène les chunks les plus proches)
        let context = similar_chunks
            .iter()
            .map(|c| format!("{}\n\n", c.text))
            .collect::<String>();

        let prompt = format!(
            "Answer the question below using only the following code context:\n\
         ---\n\
         {context}\n\
         ---\n\
         Question: {question}\n\
         Instructions: {instructions} \n\
         Answer:"
        );

        println!("{prompt}");

        utils::calculate_ask_cost(&prompt);

        // 5. Appel au LLM pour générer la réponse augmentée
        let response = chatter::chat_mistral(prompt).await?;

        Ok(response)
    })
}
