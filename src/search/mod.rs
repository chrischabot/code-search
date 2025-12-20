mod plain;
mod regex_search;
pub mod semantic;

use anyhow::Result;
use std::path::Path;

use crate::cli::{SearchArgs, SearchMode};
use crate::output::{self, SearchResult};

pub use plain::PlainSearcher;
pub use regex_search::RegexSearcher;
pub use semantic::SemanticSearcher;

pub fn run_search(args: SearchArgs) -> Result<()> {
    // Set thread count if specified
    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()
            .ok();
    }

    let results: Vec<SearchResult> = match args.mode {
        SearchMode::Plain => {
            let searcher = PlainSearcher::new(&args)?;
            searcher.search(&args.path)?
        }
        SearchMode::Regex => {
            let searcher = RegexSearcher::new(&args)?;
            searcher.search(&args.path)?
        }
        SearchMode::Semantic => {
            let searcher = SemanticSearcher::new(&args)?;
            searcher.search(&args.path)?
        }
    };

    // Apply max_results limit
    let results = if let Some(max) = args.max_results {
        results.into_iter().take(max).collect()
    } else {
        results
    };

    // Output results
    output::display_results(&results, &args)?;

    Ok(())
}

/// Trait for all search implementations
pub trait Searcher {
    fn search(&self, path: &Path) -> Result<Vec<SearchResult>>;
}