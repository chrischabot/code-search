use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;
use regex::bytes::Regex as BytesRegex;
use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::cli::SearchArgs;
use crate::output::SearchResult;
use crate::walker::{walk_files, WalkerConfig};

use super::Searcher;

pub struct RegexSearcher {
    pattern: BytesRegex,
    before_context: usize,
    after_context: usize,
    files_only: bool,
    count_only: bool,
    max_results: Option<usize>,
    walker_config: WalkerConfig,
}

impl RegexSearcher {
    pub fn new(args: &SearchArgs) -> Result<Self> {
        let pattern = regex::bytes::RegexBuilder::new(&args.pattern)
            .case_insensitive(args.ignore_case)
            .multi_line(true)
            .size_limit(10_000_000)
            .dfa_size_limit(10_000_000)
            .build()?;

        let (before, after) = if let Some(ctx) = args.context {
            (ctx, ctx)
        } else {
            (args.before_context, args.after_context)
        };

        Ok(Self {
            pattern,
            before_context: before,
            after_context: after,
            files_only: args.files_only,
            count_only: args.count,
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

    fn search_file(&self, path: &Path, found_count: &AtomicUsize) -> Result<Vec<SearchResult>> {
        // Early exit check
        if let Some(max) = self.max_results {
            if found_count.load(Ordering::Relaxed) >= max {
                return Ok(vec![]);
            }
        }

        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return Ok(vec![]),
        };

        let metadata = file.metadata()?;
        if metadata.len() > 100_000_000 {
            return Ok(vec![]);
        }

        let mmap = match unsafe { Mmap::map(&file) } {
            Ok(m) => m,
            Err(_) => return Ok(vec![]),
        };

        let content = &mmap[..];

        // For files_only and count_only, skip binary check
        if self.files_only {
            if self.pattern.is_match(content) {
                found_count.fetch_add(1, Ordering::Relaxed);
                return Ok(vec![SearchResult {
                    path: path.to_path_buf(),
                    line_number: 0,
                    column: 0,
                    line_content: String::new(),
                    context_before: vec![],
                    context_after: vec![],
                    score: None,
                }]);
            }
            return Ok(vec![]);
        }

        if self.count_only {
            let count = self.pattern.find_iter(content).count();
            if count > 0 {
                found_count.fetch_add(1, Ordering::Relaxed);
                return Ok(vec![SearchResult {
                    path: path.to_path_buf(),
                    line_number: count,
                    column: 0,
                    line_content: String::new(),
                    context_before: vec![],
                    context_after: vec![],
                    score: None,
                }]);
            }
            return Ok(vec![]);
        }

        // Full mode - need binary check
        let check_len = std::cmp::min(8192, mmap.len());
        if mmap[..check_len].contains(&0) {
            return Ok(vec![]);
        }

        // Quick check for any matches
        if !self.pattern.is_match(content) {
            return Ok(vec![]);
        }

        // Only calculate line_starts when we have matches
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(memchr::memchr_iter(b'\n', content).map(|i| i + 1))
            .collect();

        let get_line_num = |byte_offset: usize| -> usize {
            match line_starts.binary_search(&byte_offset) {
                Ok(n) => n,
                Err(n) => n.saturating_sub(1),
            }
        };

        let mut seen_lines = std::collections::HashSet::new();
        let mut results = Vec::new();

        for mat in self.pattern.find_iter(content) {
            if let Some(max) = self.max_results {
                if found_count.load(Ordering::Relaxed) >= max {
                    break;
                }
            }

            let match_start = mat.start();
            let line_num = get_line_num(match_start);
            
            if !seen_lines.insert(line_num) {
                continue;
            }

            let line_start = line_starts[line_num];
            let line_end = line_starts
                .get(line_num + 1)
                .map(|&s| s.saturating_sub(1))
                .unwrap_or(content.len());

            let line_content = String::from_utf8_lossy(&content[line_start..line_end]).into_owned();
            
            let column = if match_start >= line_start {
                match_start - line_start
            } else {
                eprintln!("Warning: match_start < line_start in {:?}", path);
                0
            };

            // Optimize context extraction
            let context_before = if self.before_context > 0 {
                (line_num.saturating_sub(self.before_context)..line_num)
                    .map(|ln| {
                        let start = line_starts[ln];
                        let end = line_starts
                            .get(ln + 1)
                            .map(|&s| s.saturating_sub(1))
                            .unwrap_or(content.len());
                        String::from_utf8_lossy(&content[start..end]).into_owned()
                    })
                    .collect()
            } else {
                Vec::new()
            };

            let context_after = if self.after_context > 0 {
                (line_num + 1..=(line_num + self.after_context).min(line_starts.len().saturating_sub(1)))
                    .filter_map(|ln| {
                        if ln >= line_starts.len() {
                            return None;
                        }
                        let start = line_starts[ln];
                        let end = line_starts
                            .get(ln + 1)
                            .map(|&s| s.saturating_sub(1))
                            .unwrap_or(content.len());
                        Some(String::from_utf8_lossy(&content[start..end]).into_owned())
                    })
                    .collect()
            } else {
                Vec::new()
            };

            found_count.fetch_add(1, Ordering::Relaxed);
            results.push(SearchResult {
                path: path.to_path_buf(),
                line_number: line_num + 1,
                column: column + 1,
                line_content,
                context_before,
                context_after,
                score: None,
            });
        }

        Ok(results)
    }
}

impl Searcher for RegexSearcher {
    fn search(&self, path: &Path) -> Result<Vec<SearchResult>> {
        let files = walk_files(path, &self.walker_config)?;
        let found_count = AtomicUsize::new(0);

        let results: Vec<SearchResult> = files
            .par_iter()
            .filter_map(|file| self.search_file(file, &found_count).ok())
            .flatten()
            .collect();

        Ok(results)
    }
}