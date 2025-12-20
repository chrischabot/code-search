use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "code-search")]
#[command(author, version, about = "Extremely fast code search with plain, regex, and semantic search")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Search for patterns in code
    Search(SearchArgs),
    /// Initialize or rebuild the BM25 index
    Init(InitArgs),
}

#[derive(Parser, Clone)]
pub struct SearchArgs {
    /// The pattern to search for
    pub pattern: String,

    /// Path to search in (defaults to current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Search mode
    #[arg(short, long, value_enum, default_value = "plain")]
    pub mode: SearchMode,

    /// Output format
    #[arg(short, long, value_enum, default_value = "human")]
    pub output: OutputFormat,

    /// Case insensitive search
    #[arg(short = 'i', long)]
    pub ignore_case: bool,

    /// Show N lines of context before matches
    #[arg(short = 'B', long, default_value = "0")]
    pub before_context: usize,

    /// Show N lines of context after matches
    #[arg(short = 'A', long, default_value = "0")]
    pub after_context: usize,

    /// Show N lines of context around matches
    #[arg(short = 'C', long)]
    pub context: Option<usize>,

    /// Filter by file extensions (e.g., py,js,rs)
    #[arg(short = 't', long, value_delimiter = ',')]
    pub file_types: Option<Vec<String>>,

    /// Filter by language (python, javascript, typescript, java, rust, go)
    #[arg(short = 'l', long, value_delimiter = ',')]
    pub languages: Option<Vec<Language>>,

    /// Maximum number of results to show
    #[arg(short = 'n', long)]
    pub max_results: Option<usize>,

    /// Show only filenames with matches
    #[arg(long)]
    pub files_only: bool,

    /// Count matches instead of showing them
    #[arg(short = 'c', long)]
    pub count: bool,

    /// Number of threads to use (0 = auto)
    #[arg(short = 'j', long, default_value = "0")]
    pub threads: usize,

    /// Include hidden files
    #[arg(long)]
    pub hidden: bool,

    /// Don't respect .gitignore
    #[arg(long)]
    pub no_ignore: bool,
}

#[derive(Parser, Clone)]
pub struct InitArgs {
    /// Path to initialize index in
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Force rebuild even if index exists
    #[arg(short, long)]
    pub force: bool,

    /// Filter by language for indexing
    #[arg(short = 'l', long, value_delimiter = ',')]
    pub languages: Option<Vec<Language>>,

    /// Number of threads to use (0 = auto)
    #[arg(short = 'j', long, default_value = "0")]
    pub threads: usize,

    /// Include hidden files
    #[arg(long)]
    pub hidden: bool,

    /// Don't respect .gitignore
    #[arg(long)]
    pub no_ignore: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SearchMode {
    /// Plain text search
    Plain,
    /// Regular expression search
    Regex,
    /// Semantic search using BM25
    Semantic,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable colored output
    Human,
    /// JSON output for machine consumption
    Json,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum, Debug)]
pub enum Language {
    Python,
    Javascript,
    Typescript,
    Java,
    Rust,
    Go,
}

impl Language {
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::Python => &["py", "pyi", "pyw"],
            Language::Javascript => &["js", "mjs", "cjs", "jsx"],
            Language::Typescript => &["ts", "tsx", "mts", "cts"],
            Language::Java => &["java"],
            Language::Rust => &["rs"],
            Language::Go => &["go"],
        }
    }

    pub fn all_extensions() -> Vec<&'static str> {
        let mut exts = Vec::new();
        for lang in [
            Language::Python,
            Language::Javascript,
            Language::Typescript,
            Language::Java,
            Language::Rust,
            Language::Go,
        ] {
            exts.extend(lang.extensions());
        }
        exts
    }
}