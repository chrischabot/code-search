use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::cli::InitArgs;
use crate::search::semantic::tokenize;
use crate::walker::{walk_files, WalkerConfig};

pub const INDEX_DIR: &str = ".code-search";
const INDEX_FILE: &str = "bm25_index.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct BM25Index {
    /// Term frequencies per document: path -> (term -> count)
    pub documents: HashMap<PathBuf, HashMap<String, u32>>,
    /// Inverse document frequencies: term -> IDF score
    pub idf: HashMap<String, f64>,
    /// Average document length
    pub avg_doc_len: f64,
    /// Total number of documents
    pub total_docs: usize,
    /// File modification times for change detection
    pub file_mtimes: HashMap<PathBuf, u64>,
}

impl BM25Index {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            idf: HashMap::new(),
            avg_doc_len: 0.0,
            total_docs: 0,
            file_mtimes: HashMap::new(),
        }
    }
}

impl Default for BM25Index {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize or update the BM25 index
pub fn init_index(args: InitArgs) -> Result<()> {
    let index_dir = args.path.join(INDEX_DIR);
    let index_path = index_dir.join(INDEX_FILE);

    // Load existing index or create new
    let mut index = if !args.force && index_path.exists() {
        eprintln!("Loading existing index...");
        load_index(&index_dir)?
    } else {
        eprintln!("Creating new index...");
        BM25Index::new()
    };

    let walker_config = WalkerConfig {
        hidden: args.hidden,
        no_ignore: args.no_ignore,
        threads: args.threads,
        extensions: None,
        languages: args.languages.clone(),
    };

    // Get all source files
    let files = walk_files(&args.path, &walker_config)?;
    eprintln!("Found {} files to index", files.len());

    // Determine which files need reindexing
    let files_to_index: Vec<PathBuf> = files
        .iter()
        .filter(|path| needs_reindex(path, &index))
        .cloned()
        .collect();

    eprintln!("{} files need indexing", files_to_index.len());

    // Remove deleted files from index
    let current_files: std::collections::HashSet<_> = files.iter().collect();
    let files_to_remove: Vec<PathBuf> = index
        .documents
        .keys()
        .filter(|p| !current_files.contains(p))
        .cloned()
        .collect();

    for path in &files_to_remove {
        index.documents.remove(path);
        index.file_mtimes.remove(path);
    }

    if !files_to_remove.is_empty() {
        eprintln!("Removed {} deleted files from index", files_to_remove.len());
    }

    // Index files in parallel
    let new_docs: Vec<(PathBuf, HashMap<String, u32>, u64)> = files_to_index
        .par_iter()
        .filter_map(|path| {
            match index_file(path) {
                Ok((term_freq, mtime)) => Some((path.clone(), term_freq, mtime)),
                Err(_) => None,
            }
        })
        .collect();

    // Update index with new documents
    for (path, term_freq, mtime) in new_docs {
        index.documents.insert(path.clone(), term_freq);
        index.file_mtimes.insert(path, mtime);
    }

    // Recalculate IDF scores
    recalculate_idf(&mut index);

    // Save index
    fs::create_dir_all(&index_dir)?;
    save_index(&index, &index_dir)?;

    eprintln!(
        "Index saved. {} documents, {} unique terms",
        index.total_docs,
        index.idf.len()
    );

    Ok(())
}

fn needs_reindex(path: &Path, index: &BM25Index) -> bool {
    let mtime = match get_mtime(path) {
        Ok(t) => t,
        Err(_) => return true,
    };

    match index.file_mtimes.get(path) {
        Some(&stored_mtime) => mtime != stored_mtime,
        None => true,
    }
}

fn get_mtime(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    let mtime = metadata
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    Ok(mtime)
}

fn index_file(path: &Path) -> Result<(HashMap<String, u32>, u64)> {
    let mtime = get_mtime(path)?;
    
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let tokens = tokenize(&content, true);
    let mut term_freq: HashMap<String, u32> = HashMap::new();

    for token in tokens {
        *term_freq.entry(token).or_insert(0) += 1;
    }

    Ok((term_freq, mtime))
}

fn recalculate_idf(index: &mut BM25Index) {
    // Calculate document frequencies
    let mut df: HashMap<String, usize> = HashMap::new();
    let mut total_len: usize = 0;

    for doc in index.documents.values() {
        for (term, count) in doc {
            *df.entry(term.clone()).or_insert(0) += 1;
            total_len += *count as usize;
        }
    }

    index.total_docs = index.documents.len();
    index.avg_doc_len = if index.total_docs > 0 {
        total_len as f64 / index.total_docs as f64
    } else {
        0.0
    };

    // Calculate IDF
    index.idf.clear();
    for (term, doc_count) in df {
        let idf_value = ((index.total_docs as f64 - doc_count as f64 + 0.5)
            / (doc_count as f64 + 0.5)
            + 1.0)
            .ln();
        index.idf.insert(term, idf_value);
    }
}

pub fn load_index(index_dir: &Path) -> Result<BM25Index> {
    let index_path = index_dir.join(INDEX_FILE);
    let file = File::open(&index_path).context("Failed to open index file")?;
    let reader = BufReader::new(file);
    let index: BM25Index = serde_json::from_reader(reader).context("Failed to parse index")?;
    Ok(index)
}

fn save_index(index: &BM25Index, index_dir: &Path) -> Result<()> {
    let index_path = index_dir.join(INDEX_FILE);
    let json = serde_json::to_string(index)?;
    let mut file = File::create(&index_path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}