use crate::types::{Chunk, ChunkType};
use crate::{chunking, config};
use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor};

pub fn chunk_rust(
    content: &str,
    file_path: &Path,
    max_chunk_size: usize,
    enc: &tiktoken_rs::CoreBPE,
) -> Result<Vec<Chunk>, String> {
    let language = tree_sitter_rust::language();

    let queries = vec![
        r#"(function_item name: (identifier) @name body: (block) @body) @item"#,
        r#"(struct_item name: (type_identifier) @name body: (field_declaration_list) @body) @item"#,
        r#"(impl_item type: (type_identifier) @name body: (declaration_list) @body) @item"#,
    ];

    parse_with_queries(&language, queries, content, file_path, max_chunk_size, enc)
}

pub fn chunk_javascript(
    content: &str,
    file_path: &Path,
    max_chunk_size: usize,
    enc: &tiktoken_rs::CoreBPE,
) -> Result<Vec<Chunk>, String> {
    let language = tree_sitter_javascript::language();

    let queries = vec![
        r#"(function_declaration name: (identifier) @name body: (statement_block) @body) @item"#,
        r#"(variable_declarator name: (identifier) @name value: (arrow_function body: (statement_block) @body)) @item"#,
        r#"(class_declaration name: (identifier) @name body: (class_body) @body) @item"#,
        r#"(method_definition name: (property_identifier) @name body: (statement_block) @body) @item"#,
    ];

    parse_with_queries(&language, queries, content, file_path, max_chunk_size, enc)
}

pub fn chunk_tsx(
    content: &str,
    file_path: &Path,
    max_chunk_size: usize,
    enc: &tiktoken_rs::CoreBPE,
) -> Result<Vec<Chunk>, String> {
    let language = tree_sitter_typescript::language_tsx();

    let queries = vec![
        r#"(function_declaration name: (identifier) @name body: (statement_block) @body) @item"#,
        r#"(variable_declarator name: (identifier) @name value: (arrow_function body: (statement_block) @body)) @item"#,
        r#"(class_declaration name: (identifier) @name body: (class_body) @body) @item"#,
        r#"(interface_declaration name: (type_identifier) @name body: (object_type) @body) @item"#,
        r#"(method_definition name: (property_identifier) @name body: (statement_block) @body) @item"#,
    ];

    parse_with_queries(&language, queries, content, file_path, max_chunk_size, enc)
}

pub fn chunk_java(
    content: &str,
    file_path: &Path,
    max_chunk_size: usize,
    enc: &tiktoken_rs::CoreBPE,
) -> Result<Vec<Chunk>, String> {
    let language = tree_sitter_java::language();

    let queries = vec![
        r#"(class_declaration name: (identifier) @name body: (class_body) @body) @item"#,
        r#"(method_declaration name: (identifier) @name body: (block) @body) @item"#,
        r#"(interface_declaration name: (identifier) @name body: (interface_body) @body) @item"#,
    ];

    parse_with_queries(&language, queries, content, file_path, max_chunk_size, enc)
}

fn parse_with_queries(
    language: &Language,
    queries: Vec<&str>,
    content: &str,
    file_path: &Path,
    max_chunk_size: usize,
    enc: &tiktoken_rs::CoreBPE,
) -> Result<Vec<Chunk>, String> {
    let mut parser = Parser::new();
    if parser.set_language(*language).is_err() {
        return Ok(vec![]);
    }

    let config = config::Config::from_env();
    let splitter = chunking::TextSplitter {
        chunk_size: config.chunk_size,
        chunk_overlap: config.chunk_overlap,
    };

    let tree = match parser.parse(content, None) {
        Some(tree) => tree,
        None => return Ok(vec![]),
    };

    let mut chunks = Vec::new();
    let mut chunk_index = 0;

    for query_str in queries {
        let query = match Query::new(*language, query_str) {
            Ok(q) => q,
            Err(_) => continue,
        };

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

        for m in matches {
            if let Some(capture) = m
                .captures
                .iter()
                .find(|c| query.capture_names()[c.index as usize] == "item")
            {
                let node = capture.node;
                let start_byte = node.start_byte();
                let end_byte = node.end_byte();
                if start_byte >= end_byte || end_byte > content.len() {
                    continue;
                }

                let chunk_text = &content[start_byte..end_byte];
                let start_line = node.start_position().row + 1;
                let end_line = node.end_position().row + 1;
                let function_name = extract_function_name(&m.captures, &query, content);
                let chunk_type = determine_chunk_type(query_str);

                if enc.encode_ordinary(chunk_text).len() > max_chunk_size {
                    let sub_chunks = split_large_chunk(
                        chunk_text,
                        file_path,
                        start_line,
                        chunk_index,
                        &function_name,
                        chunk_type.clone(),
                        &splitter,
                        enc,
                    )?;
                    chunks.extend(sub_chunks);
                } else {
                    chunks.push(Chunk {
                        path: file_path.to_string_lossy().to_string(),
                        chunk_index: chunk_index.to_string(),
                        chunk_start_line: start_line,
                        chunk_end_line: end_line,
                        text: chunk_text.to_string(),
                        function_name,
                        chunk_type,
                    });
                    chunk_index += 1;
                }
            }
        }
    }

    Ok(chunks)
}

fn split_large_chunk(
    chunk_text: &str,
    file_path: &Path,
    start_line: usize,
    chunk_index: usize,
    function_name: &Option<String>,
    chunk_type: ChunkType,
    splitter: &crate::chunking::TextSplitter,
    enc: &tiktoken_rs::CoreBPE,
) -> Result<Vec<Chunk>, String> {
    let mut sub_chunks =
        splitter.split_text(chunk_text, file_path.to_string_lossy().to_string(), enc)?;
    let mut sub_index = 0;

    for sub_chunk in &mut sub_chunks {
        sub_chunk.chunk_index = format!("{chunk_index}-{sub_index}");
        sub_chunk.function_name = function_name.clone();
        sub_chunk.chunk_type = chunk_type.clone();
        sub_chunk.chunk_start_line += start_line;
        sub_chunk.chunk_end_line += start_line;
        sub_index += 1;
    }

    Ok(sub_chunks)
}

fn determine_chunk_type(query_str: &str) -> ChunkType {
    if query_str.contains("class_declaration") {
        ChunkType::Class
    } else if query_str.contains("interface_declaration") {
        ChunkType::Interface
    } else if query_str.contains("struct_item") {
        ChunkType::Struct
    } else if query_str.contains("impl_item") {
        ChunkType::Impl
    } else if query_str.contains("method_") {
        ChunkType::Method
    } else {
        ChunkType::Function
    }
}

fn extract_function_name(
    captures: &[tree_sitter::QueryCapture],
    query: &Query,
    content: &str,
) -> Option<String> {
    captures
        .iter()
        .find(|c| query.capture_names()[c.index as usize] == "name")
        .and_then(|c| {
            let name_start = c.node.start_byte();
            let name_end = c.node.end_byte();
            if name_start < name_end && name_end <= content.len() {
                Some(content[name_start..name_end].to_string())
            } else {
                None
            }
        })
}
