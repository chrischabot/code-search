---
name: code-search
description: High-performance code search with plain text, regex, and semantic (BM25) search. Use for finding code patterns, function definitions, API usage, or semantically related code across large codebases. Particularly useful for understanding unfamiliar codebases, refactoring, and locating specific implementations.
---

# code-search

A high-performance code search tool designed for AI agents and developers to efficiently search and understand codebases.

## Overview

code-search provides three complementary search modes optimized for different discovery patterns:

1. **Plain Text Search**: Fast exact string matching for known identifiers
2. **Regex Search**: Pattern-based search for structural code patterns
3. **Semantic Search (BM25)**: Conceptual search using information retrieval ranking

## When to Use This Skill

Use code-search when you need to:

- **Locate specific code**: Find function/class definitions, API calls, or specific identifiers
- **Understand unfamiliar codebases**: Discover how features are implemented across multiple files
- **Refactor code**: Find all usages of a function, class, or pattern before making changes
- **Analyze patterns**: Identify coding patterns, naming conventions, or architectural decisions
- **Trace dependencies**: Follow how components interact across the codebase

## Search Mode Selection Guide

### Plain Text Search (Default)

**When to use:**
- You know the exact string you're looking for
- Finding function names, variable names, or specific identifiers
- Locating import statements or specific API calls
- Maximum speed is critical (8-20ms on medium repos)

**Examples:**
```bash
# Find useState hook usage
code-search search "useState" ./src

# Find specific function definition
code-search search "def calculate_total" ./app -l python

# Case-insensitive search
code-search search -i "TODO" ./src
```

**Best for:** Known identifiers, exact matches, speed-critical searches

### Regex Search

**When to use:**
- Searching for structural patterns (function signatures, class declarations)
- Finding variations of a pattern (different parameter counts, naming conventions)
- Locating code that matches a specific syntactic structure
- Need word boundaries or complex matching rules

**Examples:**
```bash
# Find all React hooks (useXxx pattern)
code-search search --mode regex 'use[A-Z][a-z]+\(' ./src -l javascript,typescript

# Find Python function definitions with parameters
code-search search --mode regex 'def\s+\w+\s*\([^)]+\)\s*:' ./app -l python

# Find all class declarations
code-search search --mode regex 'class\s+\w+' ./src
```

**Best for:** Structural patterns, syntax-based discovery, flexible matching

### Semantic Search (BM25)

**When to use:**
- Don't know exact terms but know the concept
- Finding code related to a feature or functionality
- Discovering how a concept is implemented across files
- Understanding architectural patterns

**Examples:**
```bash
# First, initialize the index for better performance
code-search init ./src

# Find authentication-related code
code-search search --mode semantic "user authentication login session security" ./src

# Find error handling patterns
code-search search --mode semantic "exception handling error logging retry" ./app

# Discover API integration code
code-search search --mode semantic "http request api call external service" ./src
```

**Best for:** Conceptual discovery, feature exploration, architectural understanding

## Key Features and Options

### Context Lines

Show surrounding code for better understanding:

```bash
# Show 3 lines before and after matches
code-search search -C 3 "function_name" ./src

# Show only lines before
code-search search -B 2 "pattern" ./src

# Show only lines after
code-search search -A 2 "pattern" ./src
```

### Language Filtering

Focus on specific languages:

```bash
# Search only Python files
code-search search "def " ./src -l python

# Search JavaScript and TypeScript
code-search search "async " ./src -l javascript,typescript

# All supported languages: python, javascript, typescript, java, rust, go
```

### Output Formats

#### Human-Readable (Default)
Colored terminal output with file grouping and context display.

#### JSON (Machine-Readable)
Structured output for programmatic processing:

```bash
code-search search --output json "pattern" ./src
```

JSON structure:
```json
{
  "total_matches": 10,
  "results": [
    {
      "path": "src/module.py",
      "line_number": 42,
      "column": 15,
      "line_content": "    def process_data(self):",
      "context_before": ["class DataProcessor:", "    def __init__(self):"],
      "context_after": ["        return self.transform()"],
      "score": 12.5
    }
  ]
}
```

### Special Modes

#### Files Only
List files containing matches without showing line details:
```bash
code-search search --files-only "pattern" ./src
```

#### Count Mode
Show number of matches per file:
```bash
code-search search -c "TODO" ./src
```

#### Limit Results
Control output size:
```bash
code-search search -n 10 "pattern" ./src  # Show top 10 results
```

## Advanced Usage Patterns

### Progressive Search Strategy

Start broad, then narrow:

```bash
# 1. Find all files containing the concept (semantic)
code-search search --mode semantic "database connection pooling" ./src --files-only

# 2. Find specific implementation (plain)
code-search search "ConnectionPool" ./src/db

# 3. Find all usage sites (regex)
code-search search --mode regex 'ConnectionPool\(' ./src -C 2
```

### Refactoring Workflow

```bash
# 1. Find all usages before changing
code-search search "old_function_name" ./src -c

# 2. Examine actual usage contexts
code-search search "old_function_name" ./src -C 3

# 3. Verify similar patterns that might need updating
code-search search --mode regex 'old_\w+_name' ./src
```

### Understanding New Codebase

```bash
# 1. Initialize semantic index
code-search init ./repo -l python,javascript

# 2. Find main entry points
code-search search --mode semantic "main entry point initialization setup" ./repo -n 5

# 3. Understand authentication
code-search search --mode semantic "authentication authorization login" ./repo -n 10

# 4. Find specific implementations
code-search search --mode regex 'class\s+\w+Auth\w+' ./repo -l python
```

## Performance Characteristics

- **Plain Search**: ~200ms on large repos (6800+ files), scales linearly
- **Regex Search**: ~200-400ms depending on pattern complexity
- **Semantic Search**: 
  - First run (no index): ~600ms build + search time
  - With index: ~50-200ms
  - Incremental reindex: Only changed files

**Optimization tips:**
- Run `code-search init` before semantic searches for best performance
- Use language filters (`-l`) to reduce files searched
- Use `--files-only` when you only need file lists
- Limit results with `-n` for faster response when you don't need all matches

## Integration with Agent Workflows

### Code Analysis

```bash
# Find all error handling in a service
code-search search --mode semantic "error exception handling logging" ./services -l python --output json
```

### Documentation Generation

```bash
# Find all public API functions
code-search search --mode regex 'export\s+function\s+\w+' ./src -l typescript -C 2

# Find documented functions
code-search search --mode regex '/\*\*[\s\S]*?\*/\s*function' ./src
```

### Testing Coverage

```bash
# Find all test files
code-search search --files-only "test" ./tests -l python

# Find untested functions (functions without corresponding tests)
# First get all functions
code-search search --mode regex 'def\s+test_\w+' ./tests -c
```

### Security Auditing

```bash
# Find potential SQL injection points
code-search search --mode regex 'execute\([^?]' ./src -l python

# Find hardcoded secrets patterns
code-search search --mode regex '["\'](?:password|secret|key)["\']:\s*["\'][^"\']+["\']' ./src

# Find authentication-related code
code-search search --mode semantic "authentication authorization access control" ./src
```

## Semantic Search Best Practices

### Query Construction

For semantic search, use multiple related terms:

**Good queries:**
- "database connection pool management" (multiple related concepts)
- "user authentication login session token" (related terms in domain)
- "error handling exception retry backoff" (implementation-focused terms)

**Less effective queries:**
- "database" (too generic, single term)
- "how to connect to database" (natural language, not code-focused)
- "fix bug" (too vague)

### Understanding BM25 Scores

Semantic search returns results with relevance scores:
- Higher scores indicate stronger relevance
- Scores are relative within a query, not absolute
- Multiple query terms increase precision

Example output:
```
src/auth/manager.py
    42:     def authenticate_user(username, password):
       [score: 15.3]
```

The score (15.3) indicates this line/file strongly matches the query terms.

## Common Patterns

### Find and Replace Preparation

```bash
# Before: Understand all usage contexts
code-search search "old_api_call" ./src -C 3 --output json > usages.json

# Review contexts, then replace
```

### API Migration

```bash
# Find old API patterns
code-search search --mode regex 'OldAPI\.\w+\(' ./src

# Find semantic concepts that might use different naming
code-search search --mode semantic "old api legacy deprecated" ./src
```

### Code Review Preparation

```bash
# Find all changed patterns in pull request
code-search search --mode semantic "new feature implementation" ./src -l python -n 20

# Verify conventions are followed
code-search search --mode regex 'async\s+def\s+\w+' ./src -l python -c
```

## Troubleshooting

### No results found

1. **Check file paths**: Ensure the path is correct
2. **Verify language filter**: Remove `-l` flag to search all files
3. **Check .gitignore**: Use `--no-ignore` to search ignored files
4. **Try broader search**: Use semantic mode with related terms

### Slow performance

1. **Initialize index**: Run `code-search init` for semantic searches
2. **Use language filters**: Add `-l` to reduce files searched
3. **Limit results**: Add `-n 100` to stop after finding enough matches
4. **Check repo size**: Very large repos (>50k files) may be slow

### Semantic search not finding relevant code

1. **Reinitialize index**: Run `code-search init --force`
2. **Use more specific terms**: Add domain-specific terminology
3. **Try multiple queries**: Different term combinations reveal different code
4. **Fall back to regex**: If you know structural patterns, regex may be better

## Reference

### All Options

```
Search Options:
  -m, --mode <MODE>              Search mode: plain, regex, semantic [default: plain]
  -o, --output <FORMAT>          Output format: human, json [default: human]
  -i, --ignore-case              Case-insensitive search
  -B, --before-context <N>       Lines of context before matches
  -A, --after-context <N>        Lines of context after matches
  -C, --context <N>              Lines of context around matches
  -t, --file-types <TYPES>       Filter by file extensions (comma-separated)
  -l, --languages <LANGS>        Filter by language (comma-separated)
  -n, --max-results <N>          Maximum number of results
  --files-only                   Show only filenames with matches
  -c, --count                    Count matches instead of showing them
  -j, --threads <N>              Number of threads (0 = auto)
  --hidden                       Include hidden files
  --no-ignore                    Don't respect .gitignore

Index Options:
  -f, --force                    Force rebuild of index
  -l, --languages <LANGS>        Filter by language for indexing
  -j, --threads <N>              Number of threads (0 = auto)
  --hidden                       Include hidden files
  --no-ignore                    Don't respect .gitignore
```

### Supported Languages

- Python: `.py`, `.pyi`, `.pyw`
- JavaScript: `.js`, `.mjs`, `.cjs`, `.jsx`
- TypeScript: `.ts`, `.tsx`, `.mts`, `.cts`
- Java: `.java`
- Rust: `.rs`
- Go: `.go`

## Installation

Build from source in the code-search directory:

```bash
cd code-search
cargo build --release

# Binary will be at target/release/code-search
```

## Agent Integration Tips

### JSON Output Parsing

When integrating with other tools, use JSON output:

```bash
results=$(code-search search --output json "pattern" ./src)
```

Parse the JSON to extract specific information:
- `total_matches`: Number of results found
- `results[].path`: File paths
- `results[].line_number`: Line numbers
- `results[].line_content`: Actual matching lines
- `results[].score`: Relevance score (semantic search only)

### Batch Processing

For multiple searches, use separate invocations:

```bash
# Find all components
code-search search --mode regex 'class\s+\w+Component' ./src -l typescript --files-only

# Then analyze each file individually
for file in $(previous_results); do
    code-search search "render" "$file" -C 5
done
```

### Combining with Other Tools

```bash
# Find files, then analyze with another tool
files=$(code-search search --mode semantic "api endpoint" ./src --files-only)

# Use grep for post-filtering
code-search search "function" ./src | grep -A 2 "async"

# Export for external analysis
code-search search --output json "TODO" ./src > todos.json
```

## Version History

### 0.1.0 (Current)
- Initial release with plain, regex, and BM25 semantic search
- Support for 6 programming languages
- JSON and human-readable output formats
- Incremental BM25 indexing with file change detection
- Performance: 75-89% of ripgrep speed on large repositories