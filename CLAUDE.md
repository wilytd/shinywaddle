# CLAUDE.md

## Project Overview

**shinywaddle** (working name: `fs-cleaner`) is a Linux filesystem de-duplicator and structure flattener. It detects and safely resolves redundant nested directory structures (e.g., `/home/app/app/src` -> `/home/app/src`).

- **Language**: Rust (edition 2024)
- **License**: MIT
- **Status**: Initial implementation — core modules scaffolded with working CLI
- **Repository**: `wilytd/shinywaddle`

## Repository Structure

```
shinywaddle/
├── Cargo.toml          # Package manifest and dependencies
├── src/
│   ├── main.rs         # CLI entry point (clap subcommands)
│   ├── lib.rs          # Crate root — re-exports modules
│   ├── analyzer.rs     # Detect redundant nesting patterns
│   ├── scanner.rs      # Pre-move risk scanning (collisions, symlinks)
│   ├── mover.rs        # Safe flatten operations with dry-run support
│   ├── journal.rs      # Rollback journal (JSON-serialized move records)
│   └── error.rs        # Error types (thiserror)
├── README.md           # Full project specification
├── LICENSE             # MIT
├── CLAUDE.md           # This file
└── .gitignore          # Excludes /target
```

## Build & Run

```bash
# Build
cargo build

# Run the CLI
cargo run -- analyze <path>
cargo run -- apply <path> --dry-run
cargo run -- apply <path>
cargo run -- rollback <path>
cargo run -- report <path>

# Verbose logging
cargo run -- -v analyze <path>
```

## Testing

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test analyzer
cargo test scanner
cargo test mover
cargo test journal
```

Tests use `tempfile` for temporary directory fixtures. All filesystem operations are tested against isolated temp dirs — never against real user data.

Current test coverage:
- `analyzer` — nesting detection, no-nesting case
- `scanner` — collision detection, symlink risk detection, clean-parent case
- `mover` — dry-run safety, apply moves, collision abort
- `journal` — save/load roundtrip, rollback reversal

## Linting & Formatting

```bash
# Lint with clippy (must pass with zero warnings)
cargo clippy

# Check formatting
cargo fmt -- --check

# Auto-format
cargo fmt
```

All code must pass `cargo clippy` with no warnings and `cargo fmt -- --check` before committing.

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` (derive) | CLI argument parsing with subcommands |
| `serde` + `serde_json` | Serialization for journal and JSON reports |
| `thiserror` | Ergonomic error type definitions |
| `walkdir` | Recursive directory traversal |
| `log` + `env_logger` | Structured logging |
| `tempfile` (dev) | Temporary directories for tests |
| `assert_cmd` (dev) | CLI integration testing |
| `predicates` (dev) | Test assertion helpers |

## Architecture

### Module Responsibilities

- **`analyzer`** — `detect_nesting(path)` walks a directory and identifies `X/X/...` patterns. Returns `Vec<NestingCandidate>` describing each redundant nesting found.
- **`scanner`** — `scan(candidate)` checks a `NestingCandidate` for collision risks and symlink risks *before* any moves happen. Returns a `ScanReport`.
- **`mover`** — `flatten(candidate, dry_run)` executes (or simulates) the move. Checks the scanner first and aborts on collisions. Returns `MoveResult` with records of what moved.
- **`journal`** — `Journal` persists move records to `.fs-cleaner-journal.json`. Supports `save()`, `load()`, and `rollback()` (LIFO reversal).
- **`error`** — Central `Error` enum with variants for I/O, collisions, permissions, broken symlinks, and cross-device moves.

### Data Flow

```
analyze/apply <path>
  → analyzer::detect_nesting()     → Vec<NestingCandidate>
  → scanner::scan()                → ScanReport (collisions, symlink risks)
  → mover::flatten(dry_run?)       → MoveResult (records of moves)
  → journal::save()                → .fs-cleaner-journal.json
```

### CLI Subcommands

| Command | Description |
|---------|-------------|
| `analyze <path>` | Detect and report nesting, show proposed moves and risks |
| `apply <path>` | Execute flattening (use `--dry-run` for simulation) |
| `rollback <path>` | Reverse a previous apply using the saved journal |
| `report <path>` | Output JSON report for automation |

## Development Principles

These are the project's design constraints — follow them when writing code:

1. **Safe by default** — dry-run first, never move files blindly
2. **No destructive operations without explicit approval**
3. **Everything reversible** when possible
4. **Dependency-aware** — scan for path references before moving
5. **Collision-safe** — abort on name collisions, never overwrite
6. **Metadata-preserving** — permissions, ownership, timestamps
7. **Prefer warnings over silent behavior**
8. **Deterministic output** — same input produces same result
9. **Idempotent** — repeatable without damage
10. **Minimal external dependencies**

## Safety Model

| Risk | Required Behavior |
|------|-------------------|
| Name collision | Abort |
| Symlink break | Warn / optional fix |
| Hardcoded path detected | Warn |
| Permission issue | Abort |
| Cross-device move | Safe copy + verify |
| Unknown file type | Skip |

## Code Conventions

- Use `thiserror` for error types — all errors go through `error::Error`
- Prefer `&Path` over `&PathBuf` in function signatures
- Use `log` macros (`info!`, `warn!`, `error!`) for operational output
- Use `println!` only for user-facing CLI output
- Unit tests live in `#[cfg(test)] mod tests` inside each module
- Test filesystem operations against `tempfile::TempDir` — never real paths
- Keep modules focused: one responsibility per file

## Contributing Guidelines

- Read `README.md` for the full project specification before making changes
- Run `cargo clippy`, `cargo fmt -- --check`, and `cargo test` before committing
- All filesystem operations must have dry-run support
- Never introduce silent destructive behavior
- Log all actions for auditability
- Handle edge cases listed in README (symlinks outside tree, Docker bind mounts, git repos inside nested dirs, cross-user permissions, network mounts)

## Non-Goals

Do not implement or scope-creep into:

- Backup tool functionality
- Full dependency resolution
- Package management
- Hash-based file deduplication
- Fixing broken application configs
