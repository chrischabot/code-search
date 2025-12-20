# Release Process

This document describes how to create a new release of code-search.

## Automated Releases (Recommended)

The project uses GitHub Actions for automated multi-platform builds.

### Creating a Release

1. Update the version in `Cargo.toml` and `VERSION` file
2. Commit the version bump: `git commit -am "chore: bump version to X.Y.Z"`
3. Create and push a tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
4. GitHub Actions will automatically:
   - Build binaries for Linux x86_64, macOS x86_64, and macOS ARM64
   - Create a GitHub release
   - Upload all binaries to the release

## Manual Release (Linux only)

For manual Linux builds:

```bash
# Build the Linux release
bash build-release.sh

# The tarball will be in: release/code-search-vX.Y.Z-linux-x86_64.tar.gz
```

## Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update version in `VERSION` file
- [ ] Update `CHANGELOG.md` (if exists)
- [ ] Run tests: `cargo test --release`
- [ ] Update README if needed
- [ ] Commit version bump
- [ ] Create and push tag
- [ ] Verify GitHub Actions build succeeds
- [ ] Download and test release binaries
- [ ] Update release notes if needed

## Platform Support

### Tier 1 (Automated builds)
- Linux x86_64 (GNU)
- macOS x86_64 (Intel)
- macOS ARM64 (Apple Silicon)

### Tier 2 (Manual builds possible)
- Linux ARM64
- Windows x86_64 (requires separate toolchain)

## Cross-Compilation Notes

### For macOS from Linux
Cross-compiling for macOS from Linux requires osxcross or similar tools, which is complex to set up. The GitHub Actions workflow handles this automatically on macOS runners.

### For Windows from Linux
Requires mingw-w64 toolchain:
```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```