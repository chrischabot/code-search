use anyhow::Result;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::cli::SearchArgs;
use crate::index::{load_index, BM25Index, INDEX_DIR};
use crate::output::SearchResult;
use crate::walker::{walk_files, WalkerConfig};

use super::Searcher;

pub struct SemanticSearcher {
    query_tokens: Vec<String>,
    ignore_case: bool,
    before_context: usize,
    after_context: usize,
    max_results: Option<usize>,
    walker_config: WalkerConfig,
}

impl SemanticSearcher {
    pub fn new(args: &SearchArgs) -> Result<Self> {
        let query_tokens = tokenize(&args.pattern, args.ignore_case);
        
        let (before, after) = if let Some(ctx) = args.context {
            (ctx, ctx)
        } else {
            (args.before_context, args.after_context)
        };

        Ok(Self {
            query_tokens,
            ignore_case: args.ignore_case,
            before_context: before,
            after_context: after,
            max_results: args.max_results,
            walker_config: WalkerConfig {
                hidden: args.hidden,
                no_ignore: args.no_ignore,
                threads: args.threads,
                extensions: args.file_types.clone(),
                languages: args.languages.clone(),
            },
        })
    }
}

impl Searcher for SemanticSearcher {
    fn search(&self, path: &Path) -> Result<Vec<SearchResult>> {
        // Try to load existing index, or create one on-the-fly
        let index_path = path.join(INDEX_DIR);
        let index = if index_path.exists() {
            match load_index(&index_path) {
                Ok(idx) => idx,
                Err(_) => {
                    eprintln!("Warning: Failed to load index, building on-the-fly...");
                    build_index_on_the_fly(path, &self.walker_config)?
                }
            }
        } else {
            eprintln!("No index found. Building on-the-fly (run 'code-search init' for faster searches)...");
            build_index_on_the_fly(path, &self.walker_config)?
        };

        // Calculate BM25 scores for each document
        let mut scored_docs: Vec<(PathBuf, f64)> = index
            .documents
            .par_iter()
            .map(|(path, doc_stats)| {
                let score = calculate_bm25(
                    &self.query_tokens,
                    doc_stats,
                    &index.idf,
                    index.avg_doc_len,
                    index.total_docs,
                );
                (path.clone(), score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        // Sort by score descending
        scored_docs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        if let Some(max) = self.max_results {
            scored_docs.truncate(max);
        }

        // For each matching document, find the best matching line
        let mut results: Vec<SearchResult> = Vec::new();

        for (doc_path, score) in scored_docs {
            if let Ok(matches) = self.find_best_lines(&doc_path, score) {
                results.extend(matches);
            }
        }

        Ok(results)
    }
}

impl SemanticSearcher {
    fn find_best_lines(&self, path: &Path, doc_score: f64) -> Result<Vec<SearchResult>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();

        let query_set: std::collections::HashSet<&str> = 
            self.query_tokens.iter().map(|s| s.as_str()).collect();

        let mut line_scores: Vec<(usize, f64)> = lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let line_tokens = tokenize(line, self.ignore_case);
                let mut score = 0.0;
                for token in &line_tokens {
                    if query_set.contains(token.as_str()) {
                        score += 1.0;
                    }
                }
                if score > 0.0 {
                    Some((idx, score))
                } else {
                    None
                }
            })
            .collect();

        line_scores.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top_lines: Vec<(usize, f64)> = line_scores.into_iter().take(5).collect();

        let mut results = Vec::with_capacity(top_lines.len());
        for (line_idx, line_score) in top_lines {
            let line_content = lines[line_idx].clone();
            
            let context_before = if self.before_context > 0 {
                (line_idx.saturating_sub(self.before_context)..line_idx)
                    .filter_map(|i| lines.get(i).cloned())
                    .collect()
            } else {
                Vec::new()
            };

            let context_after = if self.after_context > 0 {
                (line_idx + 1..=(line_idx + self.after_context).min(lines.len().saturating_sub(1)))
                    .filter_map(|i| lines.get(i).cloned())
                    .collect()
            } else {
                Vec::new()
            };

            results.push(SearchResult {
                path: path.to_path_buf(),
                line_number: line_idx + 1,
                column: 1,
                line_content,
                context_before,
                context_after,
                score: Some(doc_score * line_score),
            });
        }

        Ok(results)
    }
}

/// Tokenize text for BM25 - optimized version
pub fn tokenize(text: &str, lowercase: bool) -> Vec<String> {
    // Pre-allocate with reasonable capacity
    let mut tokens = Vec::with_capacity(text.len() / 10);
    
    let text_cow = if lowercase {
        std::borrow::Cow::Owned(text.to_lowercase())
    } else {
        std::borrow::Cow::Borrowed(text)
    };
    
    // Split and filter in one pass
    for token in text_cow.split(|c: char| !c.is_alphanumeric() && c != '_') {
        if token.len() >= 2 {
            tokens.push(token.to_string());
        }
    }
    
    tokens
}

/// BM25 parameters
const K1: f64 = 1.2;
const B: f64 = 0.75;

/// Calculate BM25 score
fn calculate_bm25(
    query_tokens: &[String],
    doc_stats: &FxHashMap<String, u32>,
    idf: &FxHashMap<String, f64>,
    avg_doc_len: f64,
    _total_docs: usize,
) -> f64 {
    let doc_len: u32 = doc_stats.values().sum();
    let doc_len_f64 = doc_len as f64;

    let mut score = 0.0;
    for token in query_tokens {
        if let (Some(&tf), Some(&term_idf)) = (doc_stats.get(token), idf.get(token)) {
            let tf_f64 = tf as f64;
            let numerator = tf_f64 * (K1 + 1.0);
            let denominator = tf_f64 + K1 * (1.0 - B + B * (doc_len_f64 / avg_doc_len));
            score += term_idf * (numerator / denominator);
        }
    }

    score
}

/// Build index on-the-fly (slower, used when no pre-built index exists)
fn build_index_on_the_fly(path: &Path, config: &WalkerConfig) -> Result<BM25Index> {
    use std::io::Read;
    
    let files = walk_files(path, config)?;
    
    let mut documents: FxHashMap<PathBuf, FxHashMap<String, u32>> = FxHashMap::default();
    let mut df: FxHashMap<String, usize> = FxHashMap::default();
    let mut total_len: usize = 0;

    for file_path in &files {
        if let Ok(mut file) = File::open(file_path) {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                let tokens = tokenize(&content, true);
                let mut term_freq: FxHashMap<String, u32> = FxHashMap::default();
                
                for token in &tokens {
                    *term_freq.entry(token.clone()).or_insert(0) += 1;
                }

                // Update document frequency
                for token in term_freq.keys() {
                    *df.entry(token.clone()).or_insert(0) += 1;
                }

                total_len += tokens.len();
                documents.insert(file_path.clone(), term_freq);
            }
        }
    }

    let total_docs = documents.len();
    let avg_doc_len = if total_docs > 0 {
        total_len as f64 / total_docs as f64
    } else {
        0.0
    };

    // Calculate IDF
    let mut idf: FxHashMap<String, f64> = FxHashMap::default();
    for (term, doc_count) in &df {
        let idf_value = ((total_docs as f64 - *doc_count as f64 + 0.5) / (*doc_count as f64 + 0.5) + 1.0).ln();
        idf.insert(term.clone(), idf_value);
    }

    Ok(BM25Index {
        documents,
        idf,
        avg_doc_len,
        total_docs,
        file_mtimes: FxHashMap::default(),
    })
}