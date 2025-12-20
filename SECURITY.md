# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Security Vulnerabilities Addressed

### v0.1.0 Security Fixes

The following vulnerabilities were identified and fixed in version 0.1.0:

#### CRITICAL: Regex Denial of Service (Fixed)

**Issue:** User-supplied regex patterns had no complexity limits, allowing catastrophic backtracking patterns.

**Attack Example:**
```bash
code-search search --mode regex '(a+)+b' /large/repo
```

**Mitigation:** Added regex compilation limits:
- `size_limit(10_000_000)` - Maximum compiled regex size
- `dfa_size_limit(10_000_000)` - Maximum DFA cache size

#### CRITICAL: Index File Size Validation (Fixed)

**Issue:** Maliciously large index files could cause out-of-memory during JSON deserialization.

**Attack Example:** Crafted 10GB `.code-search/bm25_index.json` file

**Mitigation:** Index files are now size-checked before loading (100MB limit).

#### CRITICAL: Integer Underflow in Column Calculation (Fixed)

**Issue:** Edge case in binary search could cause integer underflow leading to panic or incorrect results.

**Mitigation:** Added bounds checking with graceful fallback to column 0.

#### HIGH: Unbounded Memory Consumption (Fixed)

**Issue:** Semantic search loaded entire files into memory without size limits.

**Mitigation:** Added 100MB file size check before loading.

#### HIGH: Context Size Limits (Fixed)

**Issue:** No limits on context lines could cause memory exhaustion.

**Attack Example:**
```bash
code-search search -C 999999 'pattern' .
```

**Mitigation:** Context lines capped at 100 per side.

## Known Limitations

### Symlink Handling

The tool uses the `ignore` crate for file traversal which has specific symlink behavior:
- Symlinks are followed by default
- Could potentially access files outside the intended search directory

**Risk Level:** LOW (standard tool behavior)

**Mitigation:** Users should be aware when searching directories with symlinks.

### File System Race Conditions

Time-of-check time-of-use (TOCTOU) races exist in:
- Index file modification time checking
- File size validation before memory mapping

**Risk Level:** LOW (accepted for performance)

**Mitigation:** These are inherent to file system operations and acceptable for this tool's threat model.

## Security Best Practices

### For Users

1. **Don't search untrusted directories:**
   - Malicious files (symlinks, large files) could impact performance
   - Always verify the source of codebases before searching

2. **Validate regex patterns:**
   - While protected against DoS, complex patterns can still be slow
   - Test patterns on small directories first

3. **Be careful with index files:**
   - The `.code-search/` directory contains cached data
   - Delete if you suspect corruption: `rm -rf .code-search/`

4. **Context size awareness:**
   - Large context values (-C) can consume significant memory
   - Maximum of 100 lines enforced

### For Developers

1. **Dependency Auditing:**
   ```bash
   cargo install cargo-audit
   cargo audit
   ```

2. **Fuzzing:**
   - Regex patterns
   - File paths
   - Index files

3. **Memory Profiling:**
   - Test with extremely large repositories
   - Monitor memory usage under load

## Reporting a Vulnerability

Please report security vulnerabilities to the repository maintainers via:
- GitHub Security Advisories (preferred)
- Opening a GitHub issue (for low-severity issues)

Include:
- Description of the vulnerability
- Steps to reproduce
- Impact assessment
- Suggested fix (if available)

## Security Features

### Input Validation

- ✅ Regex complexity limits
- ✅ File size limits (100MB)
- ✅ Context size limits (100 lines)
- ✅ Binary file detection and skipping

### Resource Limits

- ✅ Maximum file size: 100MB
- ✅ Maximum context lines: 100
- ✅ Maximum regex DFA size: 10MB
- ✅ Maximum index file size: 100MB

### Error Handling

- ✅ Graceful degradation on file access errors
- ✅ No sensitive information in error messages
- ✅ Proper error propagation using Result<>

### Memory Safety

- ✅ Rust's memory safety guarantees
- ⚠️  Limited use of `unsafe` (only for memory-mapped I/O)
- ✅ Bounds checking on all indexing operations

## Security Checklist for Contributors

When contributing code, ensure:

- [ ] No new `unsafe` blocks without justification
- [ ] Input validation for user-controlled data
- [ ] Resource limits for operations that could exhaust memory/CPU
- [ ] No panics on malicious input (use Result<>)
- [ ] Error messages don't leak sensitive paths
- [ ] Dependencies are up to date (cargo update)
- [ ] Tests cover security-relevant edge cases

## Version History

### 0.1.0 (2025-12-20)
- Initial release with comprehensive security fixes
- Regex DoS protection
- Index size validation
- Memory consumption limits
- Integer overflow protection

## References

- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CWE: Common Weakness Enumeration](https://cwe.mitre.org/)