# code-search

code-search is a command-line tool for searching source code, written in Rust for coding agents and the developers working alongside them. An agent dropped into an unfamiliar repository usually knows the idea it is looking for well before it knows the identifier, which is an awkward fit for `grep`-style tools: they return every line containing the text or pattern you asked for, in no order of relevance, and leave the reader to work out which files matter. code-search keeps exact and regex matching for when you do know the name, adds BM25 keyword ranking for when you only have a few likely words, and puts all three behind one command with one JSON output format.

It runs as a single local binary with no model, server or network connection behind it, and that also bounds what the third mode can do: despite its name, `--mode semantic` involves no embeddings and ranks files only by the words they share with your query, so a file that never uses those words will not appear, however closely it relates to the idea.

## How it works

code-search walks the tree with the `ignore` crate, which skips hidden files and anything excluded by `.gitignore`. Plain and regex search then scan the files in parallel with `rayon`, memory-mapping each one and searching its bytes with the SIMD-accelerated substring finder from `memchr` or with the `regex` crate, and they only work out line numbers and context for files that contain a match.

Semantic search uses [BM25](https://en.wikipedia.org/wiki/Okapi_BM25), the keyword-relevance formula behind many search engines, under which a file scores higher when it uses the rarer words of your query often for its length. Each file is split into lowercase tokens of letters, digits and underscores, and `code-search init` saves the per-file term counts to `.code-search/bm25_index.json` inside the directory you give it; without that index, the same counts are rebuilt in memory for every search. Once the files are ranked, code-search reads them again and reports up to five lines from each, preferring the lines with the most query terms, with a score that multiplies the file's BM25 score by the number of those terms on the line.

Tokens are whole identifiers and nothing is stemmed, so `login` matches the word `login` but not `logins`, `userLogin` or `user_login`. Queries therefore work best as several words likely to appear verbatim in the code and its comments, and a known identifier is better served by plain or regex search.

## Install

You need a stable [Rust toolchain](https://rustup.rs). Building from source gets you the current code:

```bash
git clone https://github.com/chrischabot/code-search.git
cd code-search
cargo install --path .
```

That puts `code-search` in Cargo's bin directory, usually `~/.cargo/bin`. To build without installing, run `cargo build --release` and use `target/release/code-search`.

The [v0.1.0 release](https://github.com/chrischabot/code-search/releases/tag/v0.1.0) has a prebuilt binary for Apple Silicon Macs only, built before the context and index size limits landed on `main`. [RELEASING.md](RELEASING.md) treats Linux x86_64 and both macOS architectures as the automated build targets, with Linux ARM64 and Windows as manual builds.

## Usage

```bash
# Exact text, when you know the identifier
code-search search "calculate_total" ./src -l python

# A regular expression, when you know the shape of the code
code-search search --mode regex 'use[A-Z][a-z]+\(' ./src -l javascript,typescript -C 2

# Ranked keywords, when you only have likely words
code-search init ./src
code-search search --mode semantic -i "authentication login session" ./src -n 20
```

The path defaults to the current directory, and `code-search search --help` lists every option. These are the ones you will use most:

| Option | Effect |
| --- | --- |
| `-m`, `--mode` | `plain` (default), `regex` or `semantic` |
| `-o`, `--output` | `human` (default) or `json` |
| `-i`, `--ignore-case` | Case-insensitive matching |
| `-A`, `-B`, `-C` | Lines of context after, before or around each match, up to 100 |
| `-l`, `--languages` | Comma-separated: `python`, `javascript`, `typescript`, `java`, `rust`, `go` |
| `-t`, `--file-types` | Comma-separated file extensions, used when `-l` is absent |
| `-n`, `--max-results` | Maximum number of results |
| `-c`, `--count` | Count occurrences per file (plain and regex) |
| `--files-only` | List matching files (plain and regex) |
| `-j`, `--threads` | Search threads; 0, the default, uses all available cores |
| `--hidden` | Include hidden files |
| `--no-ignore` | Also search files excluded by `.gitignore` and similar rules |

Without `-l` or `-t`, every text file that isn't hidden or ignored is searched, so the six languages are shorthand for groups of extensions and not a limit on what code-search can read.

### Working with the index

Semantic search has a few rules that follow from how its index is built and stored:

- Pass `-i`, as in the example above. The index is always lowercase, so without `-i` a capitalised query term matches nothing, and the step that picks lines only accepts the query's exact case, which can drop a file that ranked well.
- Run `code-search init <path>` again after changing files, because search trusts the index as it stands. `init` re-reads files whose modification time changed, drops deleted ones and recalculates the term weights, while `--force` rebuilds from scratch.
- Choose your filters when you run `init`, for example `code-search init ./repo -l python,javascript`. Once an index exists, semantic search scores everything in it, and `-l`, `-t`, `--hidden` and `--no-ignore` have no effect.
- Search the same path from the same directory. The index is looked up under the search path and records file paths as they were walked, so after a relative `init`, a search from another directory finds files it cannot reopen and silently leaves them out. An absolute path avoids this.
- Add `.code-search/` to your `.gitignore`.

In this mode `-n` caps both the files considered and the lines printed, so `-n 5` can return five lines from a single file.

### JSON output

`--output json` prints a single document to stdout and keeps progress messages on stderr, so the output can be parsed directly:

```json
{
  "total_matches": 1,
  "results": [
    {
      "path": "./src/auth.py",
      "line_number": 42,
      "column": 1,
      "line_content": "def authenticate(username, password):",
      "score": 9.52
    }
  ]
}
```

`score` appears only in semantic mode, and `context_before` and `context_after` only when there are context lines to show. `total_matches` counts the results in the document, after any `-n` limit. With `-c`, each result is a file with its count in `line_number`, and with `--files-only`, `line_number` is 0. The exit status is 0 whether or not anything matched, so check `total_matches` rather than the exit code. Human output does no terminal detection, so its colour codes can survive a pipe unless `NO_COLOR` is set, which is one more reason to parse JSON.

### With an agent

[SKILL.md](SKILL.md) is an agent skill covering when to use each mode, with worked examples. Once the binary is on your `PATH`, copy the file into your agent's skills directory; in Claude Code that means `~/.claude/skills/code-search/SKILL.md`.

## Technical details

Regex mode uses the syntax of the Rust `regex` crate, which has no look-around or backreferences. Symbolic links are not followed. Plain and regex search skip files over 100 MB and, unless you pass `-c` or `--files-only`, binary files, which they recognise by a NUL byte near the start of the file. Semantic search indexes only valid UTF-8.

Rough timings recorded for 0.1.0 on four repositories of 800 to 9,400 files had ripgrep ahead every time, at 15 to 300 ms against 20 to 450 ms for code-search. If all you need is exact or regex text search, ripgrep is the better tool, and the reason to use code-search is having ranked search behind the same command and output format.

## Development

```bash
cargo build --release
cargo test --release
```

The integration tests in `tests/integration_tests.rs` run the binary at `target/release/code-search`, so they need the release profile. Pushing a `v*` tag runs the release workflow, [RELEASING.md](RELEASING.md) describes the process, and `build-release.sh` packages a Linux x86_64 tarball by hand. [SECURITY.md](SECURITY.md) explains how to report a vulnerability.

## Project background

- [code-search project overview](https://chabot.dev/projects/#code-search) on Chris Chabot’s site.
- [BM25 before grep](https://chabot.dev/blog/bm25-before-grep/): using lexical ranking to find the vocabulary for an exact search.

## License and credits

MIT, copyright 2025 Chris Chabot; see [LICENSE](LICENSE). The implementation commits are authored by [Maestro](https://github.com/iGentMaestro) and merged by Chris Chabot. code-search builds on [`ignore`](https://crates.io/crates/ignore), [`regex`](https://crates.io/crates/regex), [`memchr`](https://crates.io/crates/memchr), [`rayon`](https://crates.io/crates/rayon), [`memmap2`](https://crates.io/crates/memmap2), [`clap`](https://crates.io/crates/clap), [`serde_json`](https://crates.io/crates/serde_json) and [`termcolor`](https://crates.io/crates/termcolor).
