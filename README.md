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
