use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

use crate::cli::Language;

pub struct WalkerConfig {
    pub hidden: bool,
    pub no_ignore: bool,
    pub threads: usize,
    pub extensions: Option<Vec<String>>,
    pub languages: Option<Vec<Language>>,
}

impl Default for WalkerConfig {
    fn default() -> Self {
        Self {
            hidden: false,
            no_ignore: false,
            threads: 0,
            extensions: None,
            languages: None,
        }
    }
}

pub fn walk_files<P: AsRef<Path>>(path: P, config: &WalkerConfig) -> Result<Vec<PathBuf>> {
    let mut builder = WalkBuilder::new(path.as_ref());
    
    builder
        .hidden(!config.hidden)
        .ignore(!config.no_ignore)
        .git_ignore(!config.no_ignore)
        .git_global(!config.no_ignore)
        .git_exclude(!config.no_ignore);

    if config.threads > 0 {
        builder.threads(config.threads);
    }

    // Collect allowed extensions
    let allowed_extensions: Option<Vec<&str>> = if let Some(ref langs) = config.languages {
        let mut exts = Vec::new();
        for lang in langs {
            exts.extend(lang.extensions());
        }
        Some(exts)
    } else if let Some(ref exts) = config.extensions {
        Some(exts.iter().map(|s| s.as_str()).collect())
    } else {
        None
    };

    let mut files = Vec::new();
    
    for result in builder.build() {
        let entry = result?;
        
        // Skip directories
        if entry.file_type().map_or(false, |ft| ft.is_file()) {
            let path = entry.path();
            
            // Filter by extension if specified
            if let Some(ref allowed) = allowed_extensions {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if !allowed.contains(&ext) {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

/// Walk files in parallel and apply a function to each
pub fn walk_files_parallel<P, F>(path: P, config: &WalkerConfig, f: F) -> Result<()>
where
    P: AsRef<Path>,
    F: Fn(PathBuf) + Send + Sync,
{
    use rayon::prelude::*;
    
    let files = walk_files(path, config)?;
    files.into_par_iter().for_each(f);
    
    Ok(())
}

/// Check if a path has a supported source code extension
pub fn is_source_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        Language::all_extensions().contains(&ext)
    } else {
        false
    }
}

/// Get the language for a file based on its extension
pub fn detect_language(path: &Path) -> Option<Language> {
    let ext = path.extension()?.to_str()?;
    
    for lang in [
        Language::Python,
        Language::Javascript,
        Language::Typescript,
        Language::Java,
        Language::Rust,
        Language::Go,
    ] {
        if lang.extensions().contains(&ext) {
            return Some(lang);
        }
    }
    
    None
}