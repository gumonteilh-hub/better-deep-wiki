use std::{fs, path::Path};

use crate::embedding::Embedder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

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
    let chunk_file_name = repo_path.clone().replace("/", "_") + "_chunks.bin";
    if !fs::exists(&chunk_file_name).unwrap() {
        println!("Start parsing repo");
        let meta_files = parsing::parse_repo(repo_path);
        println!("{} files detected.\nStart chunking", meta_files.len());

        let splitter = chunking::TextSplitter {
            split_by: chunking::SplitBy::Line,
            chunk_size: 350,
            chunk_overlap: 100,
        };

        let writter = chunk_writter::ChunkBinWriter::create(&chunk_file_name).unwrap();
        meta_files.par_iter().for_each(|meta| {
            // println!("Chunking of file : {:?}", meta.path);
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

    let reader = chunk_writter::ChunkBinReader::<Chunk>::open(&chunk_file_name).unwrap();
    let all_chunks: Vec<_> = reader
        .map(|r| match r {
            Ok(chunk) => {
                println!("{}: {}", chunk.path, chunk.chunk_index);
                chunk
            },
            Err(e) => {
                panic!("Error deconding chunks : {e}")
            }
        })
        .collect();

    utils::calculate_cost(&all_chunks);

    println!("Start embedding");

    let embedder = embedding::MistralEmbedder::from_env(1536);
    let batches = utils::make_batches(all_chunks);

    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    rt.block_on(async {
        let mut db = vector_store::VectorStore::new("vector_db")
            .expect("Error trying to open db : 'vector_db'");
        for batch in batches {
            match embedder.embed_batch(batch).await {
                Ok(embs) => {
                    // replace by create Embeddings and insert in db
                    match db.insert_many_embeddings_bulk(&embs) {
                        Ok(_) => (),
                        Err(_) => eprintln!("Error saving vectors in db"),
                    }
                }
                Err(e) => {
                    panic!("{e}");
                }
            }
        }
    })
}
