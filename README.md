# shinywaddle
partnership coding v2
Below is a clean, practical starter README you can drop straight into a repo. It assumes a real tool will be built (not a toy script) and emphasizes safety, auditability, and dependency awareness.

---

# Linux Filesystem De-Duplicator / Structure Flattener

## Overview

This project cleans up **redundant nested directory structures** commonly found in Linux environments, such as:

```
/home/projectname/projectname/useful
/home/projectname/projectname/src
/home/projectname/projectname/docs
```

The tool safely **lifts meaningful content up one level**:

```
/home/projectname/useful
/home/projectname/src
/home/projectname/docs
```

while preventing:

* Broken symlinks
* Dependency/path breakage
* Overwrites and collisions
* Loss of metadata or permissions

This is **not a blind move tool**. It performs analysis before execution.

---

## Problem This Solves

Redundant directory nesting happens due to:

* Poor archive extraction habits (`tar` inside same-named folder)
* Git clone into pre-created directory
* Build systems generating nested output
* Manual copy/move mistakes
* Legacy migrations

Over time this creates:

* Path bloat
* Broken scripts
* Confusing structures
* Duplicate files
* Harder automation

---

## Design Goals

* Safe by default (dry-run first)
* Dependency-aware
* Collision-safe
* Metadata-preserving
* Script-friendly (CLI first)
* Idempotent (repeatable without damage)
* Auditable (logs + diff output)

---

## Planned Features

### Core

* Detect redundant directory nesting (`X/X/...`)
* Identify candidate directories for flattening
* Dry-run mode (no changes)
* Safe move operation (no overwrite)
* Preserve:

  * Permissions
  * Ownership
  * Timestamps
* Structured logging

### Safety / Validation

* Detect symlinks and repair or warn
* Scan scripts for hardcoded paths
* Detect imports / relative path dependencies
* Optional checksum verification after move
* Abort on unsafe collision
* Rollback support (planned)

### Advanced (Planned)

* Configurable depth rules
* Pattern exclusions
* Ignore lists (.cleanupignore)
* Batch mode across many projects
* JSON output for automation
* Interactive approval mode
* Dependency graph scan (Python, Bash, Node, etc.)

---

## Example

### Before

```
/home/app/app/config
/home/app/app/data
/home/app/app/logs
```

### Command

```
fs-cleaner analyze /home/app
```

### Output

```
Detected redundant nesting: /home/app/app
Proposed moves:
  config  -> /home/app/config
  data    -> /home/app/data
  logs    -> /home/app/logs

No collisions detected
No broken symlink risk detected
3 scripts reference relative paths (safe)

Run with --apply to execute
```

### Apply

```
fs-cleaner apply /home/app
```

---

## Safety Model

The tool **never moves files blindly**. It checks:

| Risk                    | Behavior            |
| ----------------------- | ------------------- |
| Name collision          | Abort               |
| Symlink break           | Warn / optional fix |
| Hardcoded path detected | Warn                |
| Permission issue        | Abort               |
| Cross-device move       | Safe copy+verify    |
| Unknown file type       | Skip                |

---

## CLI (Planned)

```
fs-cleaner analyze <path>
fs-cleaner apply <path>
fs-cleaner rollback <path>
fs-cleaner report <path> --json
```

Options:

```
--dry-run
--force
--interactive
--ignore <pattern>
--depth <n>
--verbose
```

---

## Non-Goals

* Not a backup tool
* Not a full dependency resolver
* Not a package manager
* Not a deduplication (hash-based) engine
* Not responsible for fixing broken application configs

---

## Risks & Edge Cases

* Hardcoded absolute paths in scripts
* Symlinks pointing outside tree
* Docker bind mounts referencing nested paths
* Git repos inside nested directory
* Build tools expecting current layout
* Permissions across users/groups
* Network-mounted filesystems

These will be **detected and surfaced**, not silently ignored.

---

## Implementation Ideas

Language candidates:

* Python (fastest to build, strong filesystem + parsing)
* Rust (safer, faster, harder)
* Bash (possible, harder to make safe)

Likely components:

* Directory analyzer
* Dependency/path scanner
* Safe mover (atomic where possible)
* Logger
* Rollback journal

---

## Development Principles

* No destructive operations without explicit approval
* Everything reversible when possible
* Prefer warnings over silent behavior
* Deterministic output
* Works on large trees
* Minimal external dependencies

---

## Future Direction

* System-wide cleanup mode
* CI lint for filesystem structure
* Integration with backup snapshots (ZFS/BTRFS)
* Pre-flight mode for deployment pipelines
* Visual diff / tree compare
* Automated path rewrite assistant

---

## Status

Early design / bootstrap phase.

Core analyzer and safe move engine planned first.
