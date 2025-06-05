use std::path::Path;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

mod chunking;
mod parsing;

pub fn scan_repo(repo_path: String) {
    let meta_files = parsing::parse_repo(repo_path);
    let splitter = chunking::TextSplitter {
        split_by: chunking::SplitBy::Word,
        chunk_size: 350,
        chunk_overlap: 100,
    };

    meta_files.par_iter().for_each(|meta| {
        println!("Chunking of file : {:?}", meta.path);
        let path = Path::new(&meta.path);
        match splitter.split_file(path) {
            Ok(chunks) => {
                for chunk in chunks {
                    println!("Chunk {}:\n{}\n---", chunk.index, chunk.text);
                }
            }
            Err(e) => eprintln!("Erreur sur {:?} : {}", meta.path, e),
        }
    });
}
