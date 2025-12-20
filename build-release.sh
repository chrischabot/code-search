#!/usr/bin/env bash
set -e

VERSION="0.1.0"
PROJECT_NAME="code-search"

echo "Building ${PROJECT_NAME} v${VERSION}"

# Create release directory
mkdir -p release

# Build for Linux (native)
echo "Building for Linux x86_64..."
cargo build --release

# Package Linux binary
binary_name="${PROJECT_NAME}"
platform="linux-x86_64"
output_name="${PROJECT_NAME}-v${VERSION}-${platform}"

# Copy binary
cp "target/release/${binary_name}" "release/${output_name}"

# Create tarball
cd release
tar -czf "${output_name}.tar.gz" "${output_name}"
rm "${output_name}"
cd ..

echo "Created release/${output_name}.tar.gz"

# For macOS builds, we would typically use:
# - GitHub Actions with macos-latest runners
# - Cross-compilation with osxcross (complex setup)
# - Build on actual macOS hardware

echo ""
echo "Linux release build completed:"
ls -lh release/

echo ""
echo "Note: For macOS builds, use GitHub Actions or build on macOS hardware"
echo "Example GitHub Actions target: x86_64-apple-darwin and aarch64-apple-darwin"