# ⚡ FastHooks

[![Crates.io](https://img.shields.io/crates/v/fasthooks.svg)](https://crates.io/crates/fasthooks)
[![CI](https://github.com/alfredo-baratta/fasthooks/workflows/CI/badge.svg)](https://github.com/alfredo-baratta/fasthooks/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Blazing fast Git hooks manager written in Rust.** Drop-in replacement for Husky with 27x faster startup.

```
→ Running pre-commit hook...

  ✓ eslint (3263ms)
  ✓ prettier (2045ms)
  ✓ typecheck (2507ms)

✓ 3 tasks passed
  ⏱ Completed in 3.26s
  ⚡ Saved 4.55s through parallelization
  🌱 Saved ~0.11g CO₂ vs Node.js-based tools
```

## Why FastHooks?

| Feature | FastHooks | Husky + lint-staged |
|---------|-----------|---------------------|
| Startup time | **~12ms** | ~1400ms |
| Speedup | **27x faster** | baseline |
| Language | Rust (native binary) | Node.js |
| Parallel execution | Built-in | Requires configuration |
| Dependencies | Zero runtime deps | Node.js ecosystem |
| Configuration | TOML (simple) | Multiple files |
| Carbon tracking | ✅ Built-in | ❌ |

## Table of Contents

- [Quick Start](#quick-start)
- [Commands](#commands)
- [Configuration](#configuration)
  - [Global Settings](#global-settings)
  - [Hook Configuration](#hook-configuration)
  - [Task Configuration](#task-configuration)
- [Features](#features)
  - [Glob Patterns with Exclusions](#glob-patterns-with-exclusions)
  - [Task Dependencies](#task-dependencies)
  - [Conditional Execution](#conditional-execution)
  - [Hook Arguments](#hook-arguments)
  - [Parallel Execution](#parallel-execution)
  - [Fail Fast Mode](#fail-fast-mode)
- [Migration from Husky](#migration-from-husky)
- [Performance](#performance)

## Quick Start

### Installation

```bash
# Using cargo
cargo install fasthooks

# Using Homebrew (macOS/Linux)
brew install fasthooks

# Download pre-built binary (Windows/Linux/macOS)
# See releases page for latest binaries
```

### Initialize in your project

```bash
# Create configuration and install hooks
fasthooks init

# Or migrate from Husky
fasthooks migrate
```

### Basic Configuration

Create a `fasthooks.toml` in your project root:

```toml
version = "1"

[settings]
parallel = true
fail_fast = true
show_stats = true

[hooks.pre-commit]
tasks = [
    { name = "lint", run = "npm run lint", glob = "*.{js,ts}" },
    { name = "format", run = "npm run format", glob = "*.{js,ts,json}" },
    { name = "test", run = "npm test" }
]

[hooks.pre-push]
tasks = [
    { name = "build", run = "npm run build" }
]
```

## Commands

| Command | Description |
|---------|-------------|
| `fasthooks init` | Initialize FastHooks in the current repository |
| `fasthooks install` | Install Git hooks based on configuration |
| `fasthooks uninstall` | Remove all FastHooks Git hooks |
| `fasthooks run <hook>` | Manually run a specific hook |
| `fasthooks add <hook> <cmd>` | Add a command to a hook |
| `fasthooks list` | List all configured hooks |
| `fasthooks validate` | Validate configuration file |
| `fasthooks migrate` | Migrate from Husky to FastHooks |
| `fasthooks benchmark` | Compare performance with Husky |

### Command Examples

```bash
# Run pre-commit hook manually
fasthooks run pre-commit

# Run with specific files
fasthooks run pre-commit --files src/main.rs --files src/lib.rs

# Run commit-msg hook with argument (commit message file path)
fasthooks run commit-msg -- .git/COMMIT_EDITMSG

# Run pre-push hook with arguments (remote name and URL)
fasthooks run pre-push -- origin https://github.com/user/repo.git

# Validate configuration
fasthooks validate
```

## Configuration

### Global Settings

```toml
version = "1"

[settings]
# Run tasks in parallel (default: true)
parallel = true

# Maximum parallel tasks, 0 = auto-detect CPU cores (default: 0)
max_parallel = 0

# Stop on first failure (default: true)
fail_fast = true

# Show execution statistics (default: true)
show_stats = true

# Show carbon savings estimate (default: true)
show_carbon_savings = true

# Skip all hooks in CI environment (default: false)
skip_ci = false

# Enable colored output (default: true)
colors = true
```

### Hook Configuration

```toml
[hooks.pre-commit]
# Override global parallel setting for this hook
parallel = true

# Override global fail_fast setting for this hook
fail_fast = true

# Skip this hook in CI environment
skip_ci = false

# Tasks to run
tasks = [
    # ... task definitions
]
```

### Task Configuration

Each task supports the following options:

```toml
[hooks.pre-commit]
tasks = [
    {
        # Required: Task name (for display)
        name = "lint",

        # Required: Command to execute
        run = "npm run lint",

        # Optional: Glob pattern to filter files (supports exclusions with !)
        glob = "*.{js,ts}, !*.test.js",

        # Optional: Only run on staged files (default: true)
        staged = true,

        # Optional: Working directory for the command
        cwd = "./packages/app",

        # Optional: Additional environment variables
        env = { NODE_ENV = "test" },

        # Optional: Continue even if this task fails (default: false)
        allow_failure = false,

        # Optional: Condition to run this task
        if = "branch != main",

        # Optional: Tasks that must run before this one
        depends_on = ["format", "typecheck"]
    }
]
```

## Features

### Glob Patterns with Exclusions

Filter files using glob patterns. Use `!` prefix to exclude files:

```toml
[hooks.pre-commit]
tasks = [
    # Match all .rs files except those in tests/
    { name = "clippy", run = "cargo clippy", glob = "*.rs, !tests/*.rs" },

    # Match JS/TS files except test and config files
    { name = "lint", run = "eslint", glob = "*.{js,ts}, !*.test.js, !*.config.js" },

    # Multiple patterns separated by comma or space
    { name = "format", run = "prettier --write", glob = "*.json *.md *.yml" }
]
```

**How it works:**
- Patterns are separated by comma or space
- `!pattern` excludes files matching that pattern
- Files must match at least one include pattern AND no exclude patterns
- Patterns match both filename and full path

**Examples:**
| Pattern | Matches | Doesn't Match |
|---------|---------|---------------|
| `*.rs` | `main.rs`, `src/lib.rs` | `test.js` |
| `*.rs, !tests/*.rs` | `src/main.rs` | `tests/test.rs` |
| `src/**/*.ts` | `src/app/index.ts` | `lib/index.ts` |

### Task Dependencies

Define execution order with `depends_on`:

```toml
[hooks.pre-commit]
tasks = [
    { name = "install", run = "npm install" },

    # These run after install completes
    { name = "lint", run = "npm run lint", depends_on = ["install"] },
    { name = "format", run = "npm run format", depends_on = ["install"] },

    # This runs after both lint and format complete
    { name = "test", run = "npm test", depends_on = ["lint", "format"] }
]
```

**Execution order:**
```
install
   ├── lint ──┐
   └── format ┴── test
```

**Features:**
- Tasks with dependencies wait for all dependencies to complete
- Independent tasks run in parallel (if `parallel = true`)
- Circular dependencies are detected and reported as errors

### Conditional Execution

Run tasks only when conditions are met using the `if` field:

#### Branch Conditions

```toml
[hooks.pre-commit]
tasks = [
    # Only run on main branch
    { name = "deploy-check", run = "npm run deploy:check", if = "branch == main" },

    # Skip on main branch (run on all other branches)
    { name = "dev-lint", run = "npm run lint:dev", if = "branch != main" },

    # Works with any branch name
    { name = "feature-test", run = "npm test", if = "branch == feature/new-ui" }
]
```

#### Environment Variable Conditions

```toml
[hooks.pre-commit]
tasks = [
    # Only run when CI environment variable is set
    { name = "ci-tests", run = "npm run test:ci", if = "env:CI" },

    # Only run when NOT in CI (local development)
    { name = "local-lint", run = "npm run lint:fix", if = "!env:CI" },

    # Check for any environment variable
    { name = "debug", run = "npm run debug", if = "env:DEBUG" }
]
```

#### File Existence Conditions

```toml
[hooks.pre-commit]
tasks = [
    # Only run if package-lock.json exists
    { name = "npm-audit", run = "npm audit", if = "exists:package-lock.json" },

    # Only run if .env file doesn't exist (prevent committing secrets)
    { name = "check-env", run = "echo 'Safe to commit'", if = "!exists:.env.local" },

    # Check for configuration file
    { name = "eslint", run = "eslint .", if = "exists:.eslintrc.js" }
]
```

#### Condition Reference

| Condition | Description | Example |
|-----------|-------------|---------|
| `branch == <name>` | Current branch equals name | `if = "branch == main"` |
| `branch != <name>` | Current branch not equals name | `if = "branch != develop"` |
| `env:<VAR>` | Environment variable is set | `if = "env:CI"` |
| `!env:<VAR>` | Environment variable is not set | `if = "!env:DEBUG"` |
| `exists:<path>` | File or directory exists | `if = "exists:package.json"` |
| `!exists:<path>` | File or directory doesn't exist | `if = "!exists:.secrets"` |

### Hook Arguments

Git passes arguments to certain hooks. Access them with `$1`, `$2`, etc.:

```toml
[hooks.commit-msg]
tasks = [
    # $1 = path to commit message file
    { name = "lint-commit", run = "commitlint --edit $1" },
    { name = "check-msg", run = "cat $1 | grep -E '^(feat|fix|docs):'" }
]

[hooks.pre-push]
tasks = [
    # $1 = remote name, $2 = remote URL
    { name = "check-remote", run = "echo Pushing to $1 at $2" }
]

[hooks.prepare-commit-msg]
tasks = [
    # $1 = commit msg file, $2 = source, $3 = SHA
    { name = "add-branch", run = "echo 'Branch:' $(git branch --show-current) >> $1" }
]
```

**Alternative syntax:** Use `{1}`, `{2}` instead of `$1`, `$2`:

```toml
{ name = "lint-commit", run = "commitlint --edit {1}" }
```

**Hook Arguments Reference:**

| Hook | Arguments |
|------|-----------|
| `pre-commit` | (none) |
| `prepare-commit-msg` | `$1` = commit msg file, `$2` = source, `$3` = SHA |
| `commit-msg` | `$1` = commit msg file |
| `post-commit` | (none) |
| `pre-push` | `$1` = remote name, `$2` = remote URL |
| `pre-rebase` | `$1` = upstream, `$2` = rebased branch |
| `post-checkout` | `$1` = prev HEAD, `$2` = new HEAD, `$3` = is branch checkout |
| `post-merge` | `$1` = is squash merge |

### Parallel Execution

By default, tasks run in parallel to maximize performance:

```toml
[settings]
parallel = true      # Enable parallel execution globally
max_parallel = 4     # Limit to 4 concurrent tasks (0 = auto-detect)

[hooks.pre-commit]
parallel = true      # Override for this hook

tasks = [
    # These three run simultaneously
    { name = "eslint", run = "eslint .", glob = "*.{js,ts}" },
    { name = "prettier", run = "prettier --check .", glob = "*.{json,md}" },
    { name = "stylelint", run = "stylelint '**/*.css'", glob = "*.css" }
]
```

**Sequential execution:** Set `parallel = false` or use `depends_on`:

```toml
[hooks.pre-commit]
parallel = false  # Tasks run one after another

tasks = [
    { name = "first", run = "echo 1" },
    { name = "second", run = "echo 2" },
    { name = "third", run = "echo 3" }
]
```

### Fail Fast Mode

Control behavior when a task fails:

```toml
[settings]
fail_fast = true    # Stop immediately on first failure (default)

[hooks.pre-commit]
fail_fast = false   # Override: continue running other tasks even if one fails

tasks = [
    { name = "lint", run = "npm run lint" },
    { name = "test", run = "npm test" },

    # This task continues even if it fails
    { name = "optional-check", run = "npm run check", allow_failure = true }
]
```

**Behavior comparison:**

| Setting | On Failure |
|---------|------------|
| `fail_fast = true` | Stop execution, skip remaining tasks |
| `fail_fast = false` | Continue running all tasks |
| `allow_failure = true` | Task failure doesn't affect overall result |

## Migration from Husky

FastHooks can automatically migrate your existing Husky + lint-staged configuration:

```bash
fasthooks migrate
```

This will:
1. Parse your `.husky/` directory
2. Read lint-staged config from `package.json` or `.lintstagedrc`
3. Generate equivalent `fasthooks.toml`
4. Install the new hooks

### Manual Migration Example

**Before (Husky + lint-staged):**

`.husky/pre-commit`:
```bash
#!/bin/sh
npx lint-staged
```

`package.json`:
```json
{
  "lint-staged": {
    "*.{js,ts}": ["eslint --fix", "prettier --write"],
    "*.css": "stylelint --fix"
  }
}
```

**After (FastHooks):**

`fasthooks.toml`:
```toml
version = "1"

[settings]
parallel = true
fail_fast = true

[hooks.pre-commit]
tasks = [
    { name = "eslint", run = "eslint --fix {files}", glob = "*.{js,ts}", staged = true },
    { name = "prettier", run = "prettier --write {files}", glob = "*.{js,ts}", staged = true },
    { name = "stylelint", run = "stylelint --fix {files}", glob = "*.css", staged = true }
]
```

## Configuration Validation

Validate your configuration to catch errors early:

```bash
fasthooks validate
```

**Example output for valid config:**
```
→ Validating configuration...

  Config file: /path/to/fasthooks.toml

✓ Configuration is valid!

Summary:
  • Version: 1
  • Hooks configured: 2
    → pre-commit (3 tasks)
      • lint [glob: *.{js,ts}]
      • format [depends_on: ["lint"]]
      • test [if: branch != main]
    → pre-push (1 task)
      • build

• Settings:
    Parallel: yes, Fail fast: yes, Show stats: yes
```

**Example output for invalid config:**
```
→ Validating configuration...

✗ Found 2 validation error(s):

1. Task 'build' depends on 'compile' which doesn't exist
   Location: hooks.pre-commit.tasks[2]
   Suggestion: Either add a task named 'compile' or remove it from depends_on

2. Duplicate task name: 'lint'
   Location: hooks.pre-commit.tasks[3]
   Suggestion: Each task must have a unique name within a hook
```

## Performance

Real-world benchmark on a JavaScript/TypeScript project with ESLint, Prettier, and TypeScript:

| Metric | FastHooks | Husky (via npx) | Speedup |
|--------|-----------|-----------------|---------|
| **Startup time** | 12ms | 1400ms | **117x** |
| **Hook overhead** | 52ms | 1400ms | **27x** |
| **Parallel execution** | Built-in | Manual setup | - |

Run your own benchmark:

```bash
fasthooks benchmark
```

### Carbon Savings

FastHooks estimates the environmental impact of using native tooling:

```
🌱 Saved ~0.02g CO₂ vs Node.js-based tools
```

Calculation based on:
- CPU time saved (Rust vs Node.js startup overhead)
- Average carbon intensity of electricity (475g CO₂/kWh)
- Estimated CPU power consumption (65W average)

## Supported Hooks

FastHooks supports all standard Git hooks:

| Hook | When it runs |
|------|--------------|
| `pre-commit` | Before commit is created |
| `prepare-commit-msg` | After default message, before editor |
| `commit-msg` | After commit message is entered |
| `post-commit` | After commit is created |
| `pre-push` | Before push to remote |
| `pre-rebase` | Before rebase starts |
| `post-checkout` | After checkout completes |
| `post-merge` | After merge completes |
| `pre-auto-gc` | Before automatic garbage collection |

## Complete Configuration Example

```toml
version = "1"

[settings]
parallel = true
max_parallel = 0
fail_fast = true
show_stats = true
show_carbon_savings = true
skip_ci = false
colors = true

[hooks.pre-commit]
parallel = true
fail_fast = true
tasks = [
    # Format code first
    { name = "prettier", run = "prettier --write {files}", glob = "*.{js,ts,json,md}", staged = true },

    # Lint after formatting
    { name = "eslint", run = "eslint --fix {files}", glob = "*.{js,ts}", staged = true, depends_on = ["prettier"] },

    # Type check (no file filter needed)
    { name = "typecheck", run = "tsc --noEmit", depends_on = ["prettier"] },

    # Run tests only on feature branches
    { name = "test", run = "npm test", if = "branch != main", depends_on = ["eslint", "typecheck"] },

    # Security audit in CI only
    { name = "audit", run = "npm audit", if = "env:CI", allow_failure = true }
]

[hooks.commit-msg]
tasks = [
    # Validate commit message format
    { name = "commitlint", run = "commitlint --edit $1" }
]

[hooks.pre-push]
tasks = [
    # Full test suite before push
    { name = "test-all", run = "npm run test:all" },

    # Build check
    { name = "build", run = "npm run build", depends_on = ["test-all"] }
]
```

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Built with 🦀 Rust for a greener future</sub>
</p>
