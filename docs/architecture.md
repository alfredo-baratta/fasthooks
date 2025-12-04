# Architecture

This document describes the internal architecture of FastHooks.

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                           CLI Layer                              │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │
│  │  init   │ │ install │ │   run   │ │ migrate │ │benchmark│   │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘   │
└───────┼──────────┼──────────┼──────────┼──────────┼───────────┘
        │          │          │          │          │
        ▼          ▼          ▼          ▼          ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Core Modules                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Config    │  │    Hooks    │  │         Runner          │  │
│  │  ─────────  │  │  ─────────  │  │  ─────────────────────  │  │
│  │ • Parser    │  │ • Git ops   │  │ • Task executor         │  │
│  │ • Schema    │  │ • Installer │  │ • Parallel scheduling   │  │
│  │ • Validate  │  │ • Templates │  │ • Glob matching         │  │
│  └─────────────┘  └─────────────┘  │ • Stats & carbon calc   │  │
│                                     └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
        │                   │                    │
        ▼                   ▼                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                      External Dependencies                       │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────────────┐ │
│  │  TOML   │  │  git2   │  │  tokio  │  │   glob/walkdir      │ │
│  │ (serde) │  │(libgit2)│  │ (async) │  │   (file matching)   │ │
│  └─────────┘  └─────────┘  └─────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Module Descriptions

### CLI Layer (`src/cli.rs`)

Handles command-line argument parsing using `clap`. Defines all available commands and their options.

**Key types:**
- `Cli` - Main CLI structure
- `Commands` - Enum of all subcommands

### Commands (`src/commands/`)

Each command has its own module implementing the business logic:

| Module | Command | Purpose |
|--------|---------|---------|
| `init.rs` | `fasthooks init` | Create config, install hooks |
| `install.rs` | `fasthooks install` | Install Git hooks |
| `uninstall.rs` | `fasthooks uninstall` | Remove Git hooks |
| `run.rs` | `fasthooks run` | Execute a hook manually |
| `add.rs` | `fasthooks add` | Add task to config |
| `list.rs` | `fasthooks list` | Show configured hooks |
| `migrate.rs` | `fasthooks migrate` | Import from Husky |
| `benchmark.rs` | `fasthooks benchmark` | Performance comparison |

### Config Module (`src/config/`)

Handles configuration file parsing and validation.

**Key types:**
- `Config` - Root configuration structure
- `Settings` - Global settings
- `Hook` - Hook definition
- `Task` - Task within a hook
- `HookType` - Enum of supported Git hooks

**Files:**
- `schema.rs` - Type definitions with serde
- `parser.rs` - TOML parsing logic
- `mod.rs` - Config file discovery

### Hooks Module (`src/hooks/`)

Manages Git hook installation and Git repository operations.

**Key types:**
- `GitRepository` - Wrapper around libgit2
- `HookInstaller` - Hook installation logic
- `HookTemplate` - Hook script generation

**Files:**
- `git.rs` - Git operations (staged files, branches)
- `installer.rs` - Hook file management
- `template.rs` - Shell script templates

### Runner Module (`src/runner/`)

Executes tasks with parallel support and performance tracking.

**Key types:**
- `TaskExecutor` - Main execution engine
- `TaskResult` - Result of a single task
- `HookResult` - Result of a complete hook run
- `ExecutionStats` - Timing and performance data
- `CarbonSavings` - Environmental impact calculation

**Files:**
- `executor.rs` - Parallel task execution
- `stats.rs` - Statistics and carbon calculation

## Data Flow

### Hook Execution Flow

```
1. Git triggers hook (e.g., pre-commit)
           │
           ▼
2. Hook script calls `fasthooks run pre-commit`
           │
           ▼
3. Load configuration from fasthooks.toml
           │
           ▼
4. Get staged files from Git (libgit2)
           │
           ▼
5. Filter files by glob patterns
           │
           ▼
6. Execute tasks (parallel or sequential)
           │
           ▼
7. Collect results and calculate stats
           │
           ▼
8. Display output and exit with status
```

### Configuration Loading Flow

```
1. Search for config file (fasthooks.toml, etc.)
           │
           ▼
2. Parse TOML content
           │
           ▼
3. Deserialize into Config struct
           │
           ▼
4. Apply defaults for missing values
           │
           ▼
5. Validate configuration
           │
           ▼
6. Return Config or error
```

## Design Decisions

### Why Rust?

1. **Performance** - Native binary, no runtime overhead
2. **Safety** - Memory safety without garbage collection
3. **Concurrency** - Fearless concurrency with async/await
4. **Distribution** - Single binary, easy to distribute

### Why libgit2?

Using `git2` (libgit2 bindings) instead of shelling out to `git`:

1. **Speed** - No process spawning overhead
2. **Consistency** - Same behavior across platforms
3. **Rich API** - Direct access to Git internals

### Why Tokio?

Using `tokio` for async runtime:

1. **Parallel execution** - Run tasks concurrently
2. **Efficient I/O** - Non-blocking subprocess handling
3. **Ecosystem** - Well-supported in Rust

### Configuration Format

TOML was chosen for configuration:

1. **Familiar** - Used by Cargo, widely known
2. **Readable** - Clean syntax for nested structures
3. **Type-safe** - Well-defined types for serde

## Performance Optimizations

### Startup Time

- Minimal dependencies
- No JIT or runtime initialization
- Lazy configuration loading

### Parallel Execution

- Tokio runtime for async tasks
- Semaphore-based concurrency limiting
- Independent task scheduling

### File Operations

- Glob pattern compilation
- Efficient file filtering
- Minimal filesystem access

## Testing Strategy

### Unit Tests

Located in each module as `#[cfg(test)]` blocks:
- Config parsing
- Glob matching
- Template generation

### Integration Tests

Located in `tests/integration/`:
- CLI commands
- Full hook execution
- Migration from other tools

### Benchmarks

Located in `benches/`:
- Startup time
- Config parsing
- Hook execution
