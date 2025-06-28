# Dependency Updates (June 2025)

## Updated to Latest Versions

### Core Dependencies
- **tokio**: 1.0 → 1.45
- **async-openai**: 0.19 → 0.28 (major update)
- **once_cell**: 1.19 → 1.21

### New Dependencies Added for Future Slices
- **clap**: 4.5 (CLI framework)
- **reedline**: 0.40 (REPL functionality)
- **serde_yml**: 0.0.12 (replaces deprecated serde_yaml)
- **lexopt**: 0.3 (command parsing)
- **directories**: 6.0 (cross-platform config paths)
- **git2**: 0.20 (git operations)
- **uuid**: 1.17 (unique identifiers)

### Testing Dependencies Updated
- **mockall**: 0.12 → 0.13
- **proptest**: 1.4 → 1.7
- **cargo-tarpaulin**: 0.27 → 0.32
- **tempfile**: 3.9 → 3.20
- **fake**: 2.9 → 4.3 (major update)
- **arbitrary**: 1.3 → 1.4
- **criterion**: 0.5 → 0.6
- **rstest**: 0.18 → 0.25
- **insta**: 1.36 → 1.43

## Breaking Changes to Watch For
1. **async-openai** 0.19 → 0.28 may have API changes
2. **fake** 2.9 → 4.3 is a major version bump
3. **directories** is now at v6.0 (was v5.0 in docs)

## Rust Version
- Using Rust 1.88.0 (June 2025) - Latest stable

All dependencies have been updated to their latest versions as of June 2025 for maximum compatibility and security.