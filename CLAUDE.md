# CLAUDE.md

## Project Overview

**shinywaddle** (working name: `fs-cleaner`) is a Linux filesystem de-duplicator and structure flattener. It detects and safely resolves redundant nested directory structures (e.g., `/home/app/app/src` -> `/home/app/src`).

- **License**: MIT
- **Status**: Early design / bootstrap phase — no source code implemented yet
- **Repository**: `wilytd/shinywaddle`

## Repository Structure

```
shinywaddle/
├── README.md       # Full project specification and design document
├── LICENSE          # MIT License
└── CLAUDE.md       # This file
```

The project currently contains only planning documentation. No source code, build system, tests, or CI/CD exist yet.

## Planned Architecture

The README specifies these planned components:

- **Directory analyzer** — Detect redundant nesting patterns (`X/X/...`)
- **Dependency/path scanner** — Find hardcoded paths, symlinks, imports
- **Safe mover** — Atomic move operations with collision detection
- **Logger** — Structured logging for auditability
- **Rollback journal** — Undo support for applied changes

### Planned CLI Interface

```
fs-cleaner analyze <path>      # Detect and report redundant nesting
fs-cleaner apply <path>        # Execute safe flattening
fs-cleaner rollback <path>     # Undo previous apply
fs-cleaner report <path> --json  # Machine-readable output
```

Options: `--dry-run`, `--force`, `--interactive`, `--ignore <pattern>`, `--depth <n>`, `--verbose`

## Language Decision

Not yet finalized. Candidates from the README:

| Language | Pros | Cons |
|----------|------|------|
| Python | Fast to build, strong filesystem APIs | Slower runtime |
| Rust | Safe, performant | Harder to build |
| Bash | Native to Linux | Harder to make safe |

When implementation begins, update this section with the chosen language and its associated tooling.

## Development Principles

These are the project's stated design constraints — follow them when writing code:

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

When implementing, enforce these behaviors:

| Risk | Required Behavior |
|------|-------------------|
| Name collision | Abort |
| Symlink break | Warn / optional fix |
| Hardcoded path detected | Warn |
| Permission issue | Abort |
| Cross-device move | Safe copy + verify |
| Unknown file type | Skip |

## Build & Run

No build system exists yet. When one is added, document:

- How to install dependencies
- How to build the project
- How to run the CLI
- How to run tests
- How to run linting/formatting

## Testing

No test framework exists yet. When tests are added:

- Place tests alongside or in a dedicated `tests/` directory
- Ensure filesystem operations are tested against temp directories
- Test safety checks (collisions, symlinks, permissions) thoroughly
- Include both unit tests and integration tests for CLI commands

## Contributing Guidelines

- Read `README.md` for the full project specification before making changes
- Follow the development principles listed above
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
