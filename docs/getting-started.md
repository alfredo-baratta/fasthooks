# Getting Started

This guide will help you set up FastHooks in your project.

## Prerequisites

- Git repository
- One of the following for installation:
  - Rust toolchain (`cargo`)
  - Homebrew (macOS/Linux)
  - npm/pnpm/yarn (for Node.js projects)

## Installation

### Using Cargo (Recommended)

```bash
cargo install fasthooks
```

### Using Homebrew

```bash
brew install fasthooks
```

### Using npm

For Node.js projects, you can install FastHooks as a dev dependency:

```bash
npm install -D fasthooks
```

Then add a postinstall script to `package.json`:

```json
{
  "scripts": {
    "postinstall": "fasthooks install"
  }
}
```

## Quick Setup

### Option 1: Initialize New Configuration

```bash
cd your-project
fasthooks init
```

This creates a `fasthooks.toml` with example configuration and installs the hooks.

### Option 2: Migrate from Husky

If you're already using Husky and lint-staged:

```bash
fasthooks migrate
```

This will:
1. Read your `.husky/` directory
2. Parse your lint-staged configuration
3. Generate equivalent `fasthooks.toml`
4. Install the new hooks

## Basic Configuration

After initialization, edit `fasthooks.toml`:

```toml
version = "1"

[settings]
parallel = true

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "lint"
run = "npm run lint"
glob = "*.{js,ts}"
staged = true
```

## Verify Installation

Run a hook manually to verify everything works:

```bash
fasthooks run pre-commit
```

You should see output like:

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

## Common Use Cases

### JavaScript/TypeScript Project

```toml
version = "1"

[settings]
parallel = true

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "eslint"
run = "eslint --fix"
glob = "*.{js,jsx,ts,tsx}"

[[hooks.pre-commit.tasks]]
name = "prettier"
run = "prettier --write"
glob = "*.{js,ts,json,md}"

[hooks.pre-push]
[[hooks.pre-push.tasks]]
name = "test"
run = "npm test"

[[hooks.pre-push.tasks]]
name = "build"
run = "npm run build"
```

### Rust Project

```toml
version = "1"

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "format"
run = "cargo fmt --check"

[[hooks.pre-commit.tasks]]
name = "clippy"
run = "cargo clippy -- -D warnings"

[hooks.pre-push]
[[hooks.pre-push.tasks]]
name = "test"
run = "cargo test"
```

### Python Project

```toml
version = "1"

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "black"
run = "black"
glob = "*.py"

[[hooks.pre-commit.tasks]]
name = "isort"
run = "isort"
glob = "*.py"

[[hooks.pre-commit.tasks]]
name = "flake8"
run = "flake8"
glob = "*.py"

[[hooks.pre-commit.tasks]]
name = "mypy"
run = "mypy ."
```

### Monorepo

```toml
version = "1"

[hooks.pre-commit]
parallel = true

[[hooks.pre-commit.tasks]]
name = "frontend lint"
run = "npm run lint"
cwd = "frontend"
glob = "frontend/**/*.{ts,tsx}"

[[hooks.pre-commit.tasks]]
name = "backend lint"
run = "cargo clippy"
cwd = "backend"
glob = "backend/**/*.rs"

[[hooks.pre-commit.tasks]]
name = "docs"
run = "markdownlint ."
glob = "*.md"
```

## Next Steps

- Read the [Configuration Reference](configuration.md) for all options
- Check out [Migration Guide](migration.md) for moving from other tools
- See [Troubleshooting](troubleshooting.md) for common issues

## Uninstalling

To remove FastHooks from your project:

```bash
fasthooks uninstall
```

This removes the Git hooks but keeps your `fasthooks.toml` configuration.
