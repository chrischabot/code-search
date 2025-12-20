# code-search
Extremely fast code search tool with plain, regex, and semantic (BM25) search for AI agents and developers

## Installation

```bash
# Build from source
cargo build --release

# The binary will be at target/release/code-search
```

### Pre-built Binaries

Download pre-built binaries from the [Releases](https://github.com/chrischabot/code-search/releases) page.

Available platforms:
- Linux x86_64
- macOS x86_64 (Intel)
- macOS ARM64 (Apple Silicon)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/chrischabot/code-search.git
cd code-search

# Build for your platform
cargo build --release

# The binary will be at target/release/code-search

# Or use the build script for Linux
bash build-release.sh
```

## Quick Start

```bash
# Plain text search - fastest, use when you know the exact string
code-search search "useState" ./src

# Regex search - use for patterns and structural matching
code-search search --mode regex 'use[A-Z][a-z]+\(' ./src -l javascript,typescript

# Semantic search - use for conceptual discovery
code-search init ./src                    # Build index first (recommended)
code-search search --mode semantic "authentication login user" ./src
```

## When to Use Each Search Mode

### Plain Text Search (Default)
**Best for:** Known identifiers, exact matches, maximum speed

```bash
# Find a specific function
code-search search "calculate_total" ./src -l python

# Case-insensitive search
code-search search -i "TODO" ./src

# Show context lines
code-search search -C 3 "function_name" ./src
```

### Regex Search
**Best for:** Structural patterns, variations, syntax-based discovery

```bash
# Find all React hooks (useXxx pattern)
code-search search --mode regex 'use[A-Z][a-z]+\(' ./src

# Find Python function definitions with parameters
code-search search --mode regex 'def\s+\w+\s*\([^)]+\)\s*:' ./app -l python

# Find class declarations
code-search search --mode regex 'class\s+\w+' ./src
```

### Semantic Search (BM25)
**Best for:** Conceptual discovery, feature exploration, unknown terminology

```bash
# Initialize index first for better performance
code-search init ./src

# Find authentication-related code
code-search search --mode semantic "user authentication login session security" ./src

# Find error handling patterns
code-search search --mode semantic "exception handling error logging retry" ./app

# Discover API integration code
code-search search --mode semantic "http request api call external service" ./src
```

## Common Usage Patterns

### Refactoring Workflow
```bash
# 1. Find all usages before changing
code-search search "old_function_name" ./src -c

# 2. Examine actual usage contexts
code-search search "old_function_name" ./src -C 3

# 3. Verify similar patterns
code-search search --mode regex 'old_\w+_name' ./src
```

### Understanding New Codebase
```bash
# 1. Initialize semantic index
code-search init ./repo -l python,javascript

# 2. Find main entry points
code-search search --mode semantic "main entry point initialization" ./repo -n 5

# 3. Understand specific feature
code-search search --mode semantic "authentication authorization" ./repo -n 10

# 4. Find specific implementations
code-search search --mode regex 'class\s+\w+Auth\w+' ./repo -l python
```

### Code Analysis for AI Agents
```bash
# Find all error handling (JSON output for parsing)
code-search search --mode semantic "error exception handling logging" ./services -l python --output json

# Find all public API functions
code-search search --mode regex 'export\s+function\s+\w+' ./src -l typescript -C 2

# Locate test coverage
code-search search --files-only "test" ./tests -l python
```

## Features

- **Plain Text Search**: Fast byte-level searching using SIMD-accelerated algorithms
- **Regex Search**: Full regex support with case-insensitive and multi-line modes
- **Semantic Search (BM25)**: Find conceptually relevant code using BM25 ranking
- **Multiple Output Formats**: Human-readable (colored) and JSON for machine consumption
- **Language Filtering**: Filter by 6 popular languages (Python, JavaScript, TypeScript, Java, Rust, Go)
- **Context Lines**: Show lines before/after matches
- **Parallel Processing**: Utilizes all available CPU cores
- **Git-aware**: Respects `.gitignore` by default
- **Incremental Indexing**: Only re-indexes changed files for semantic search

## Usage

### Search Commands

```bash
# Plain text search
code-search search "pattern" /path/to/code

# Regex search
code-search search --mode regex "use[A-Z][a-z]+\(" /path/to/code

# Semantic search (uses BM25 ranking)
code-search search --mode semantic "authentication login user" /path/to/code

# JSON output for machine consumption
code-search search --output json "pattern" /path/to/code

# Case-insensitive search
code-search search -i "pattern" /path/to/code

# Filter by language
code-search search -l python,rust "def " /path/to/code

# Show context lines
code-search search -C 3 "pattern" /path/to/code  # 3 lines before and after

# Count matches only
code-search search -c "pattern" /path/to/code

# Show only filenames with matches
code-search search --files-only "pattern" /path/to/code

# Limit results
code-search search -n 10 "pattern" /path/to/code
```

### Index Commands

```bash
# Initialize BM25 index (speeds up semantic search)
code-search init /path/to/code

# Force rebuild of index
code-search init --force /path/to/code

# Index only specific languages
code-search init -l python,javascript /path/to/code
```

## Options

### Search Options

| Option | Short | Description |
|--------|-------|-------------|
| `--mode` | `-m` | Search mode: `plain`, `regex`, or `semantic` |
| `--output` | `-o` | Output format: `human` or `json` |
| `--ignore-case` | `-i` | Case-insensitive search |
| `--before-context` | `-B` | Lines of context before matches |
| `--after-context` | `-A` | Lines of context after matches |
| `--context` | `-C` | Lines of context around matches |
| `--file-types` | `-t` | Filter by file extensions (comma-separated) |
| `--languages` | `-l` | Filter by language (comma-separated) |
| `--max-results` | `-n` | Maximum number of results |
| `--files-only` | | Show only filenames with matches |
| `--count` | `-c` | Count matches instead of showing them |
| `--threads` | `-j` | Number of threads (0 = auto) |
| `--hidden` | | Include hidden files |
| `--no-ignore` | | Don't respect .gitignore |

### Supported Languages

- Python (`.py`, `.pyi`, `.pyw`)
- JavaScript (`.js`, `.mjs`, `.cjs`, `.jsx`)
- TypeScript (`.ts`, `.tsx`, `.mts`, `.cts`)
- Java (`.java`)
- Rust (`.rs`)
- Go (`.go`)

## Examples

### Find all React hooks usage
```bash
code-search search --mode regex 'use[A-Z][a-z]+\(' ./src -l javascript,typescript
```

### Find authentication-related code semantically
```bash
code-search init ./src
code-search search --mode semantic "user authentication login session" ./src
```

### Get JSON output for CI integration
```bash
code-search search --output json "TODO" ./src
```

### Search with context
```bash
code-search search -C 2 "def main" ./src -l python
```

## Performance

Benchmarked against ripgrep on large repositories:

| Repository | ripgrep | code-search | Speed Ratio |
|------------|---------|-------------|-------------|
| React (6800+ files) | 194ms | 219ms | 89% |
| warehouse (1700+ files) | 40ms | 63ms | 63% |
| rustdesk (800+ files) | 15ms | 20ms | 75% |
| maven (9400+ files) | 300ms | 450ms | 67% |

## How Semantic Search Works

The semantic search mode uses BM25 (Best Matching 25), a ranking function used by search engines. It:

1. **Tokenizes** your query and the code files
2. **Calculates IDF** (Inverse Document Frequency) for each term
3. **Scores** each file based on term frequency and document length
4. **Returns** results ranked by relevance

For best performance, run `code-search init` first to pre-build the index. The index is stored in `.code-search/bm25_index.json` and automatically detects changed files.

## License

MIT
