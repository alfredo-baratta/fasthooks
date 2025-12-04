# Migration Guide

This guide covers migrating from other Git hooks tools to FastHooks.

## From Husky + lint-staged

### Automatic Migration

The fastest way to migrate is using the built-in migration command:

```bash
fasthooks migrate
```

This command:
1. Reads your `.husky/` hook scripts
2. Parses lint-staged configuration from `package.json` or `.lintstagedrc`
3. Generates an equivalent `fasthooks.toml`
4. Installs the new hooks

### Manual Migration

#### Step 1: Understand Your Current Setup

Typical Husky + lint-staged setup:

**.husky/pre-commit**
```sh
#!/usr/bin/env sh
. "$(dirname -- "$0")/_/husky.sh"

npx lint-staged
```

**package.json**
```json
{
  "lint-staged": {
    "*.{js,ts}": ["eslint --fix", "prettier --write"],
    "*.css": "stylelint --fix"
  }
}
```

#### Step 2: Create FastHooks Configuration

**fasthooks.toml**
```toml
version = "1"

[settings]
parallel = true

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "eslint"
run = "eslint --fix"
glob = "*.{js,ts}"
staged = true

[[hooks.pre-commit.tasks]]
name = "prettier"
run = "prettier --write"
glob = "*.{js,ts}"
staged = true

[[hooks.pre-commit.tasks]]
name = "stylelint"
run = "stylelint --fix"
glob = "*.css"
staged = true
```

#### Step 3: Install and Test

```bash
# Install FastHooks hooks
fasthooks install

# Test the migration
fasthooks run pre-commit
```

#### Step 4: Clean Up

```bash
# Remove Husky
rm -rf .husky
npm uninstall husky lint-staged
```

### lint-staged Pattern Mapping

| lint-staged | FastHooks |
|-------------|-----------|
| `"*.js": "eslint"` | `glob = "*.js"` |
| `"*.{js,ts}": [...]` | `glob = "*.{js,ts}"` |
| `"src/**/*.ts": "..."` | `glob = "src/**/*.ts"` |

## From Lefthook

### Configuration Comparison

**lefthook.yml**
```yaml
pre-commit:
  parallel: true
  commands:
    eslint:
      glob: "*.{js,ts}"
      run: eslint --fix {staged_files}
    prettier:
      glob: "*.{js,ts,json}"
      run: prettier --write {staged_files}
```

**fasthooks.toml**
```toml
version = "1"

[hooks.pre-commit]
parallel = true

[[hooks.pre-commit.tasks]]
name = "eslint"
run = "eslint --fix"
glob = "*.{js,ts}"
staged = true

[[hooks.pre-commit.tasks]]
name = "prettier"
run = "prettier --write"
glob = "*.{js,ts,json}"
staged = true
```

### Key Differences

| Lefthook | FastHooks |
|----------|-----------|
| `{staged_files}` | `{files}` or auto-appended |
| `commands:` | `[[hooks.X.tasks]]` |
| `scripts:` | Use `run = "./scripts/..."` |

## From pre-commit (Python)

### Configuration Comparison

**.pre-commit-config.yaml**
```yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.4.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
  - repo: https://github.com/psf/black
    rev: 23.1.0
    hooks:
      - id: black
```

**fasthooks.toml**
```toml
version = "1"

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "trailing-whitespace"
run = "sed -i 's/[[:space:]]*$//' {files}"
glob = "*"
staged = true

[[hooks.pre-commit.tasks]]
name = "black"
run = "black"
glob = "*.py"
staged = true
```

### Notes

- pre-commit downloads and manages hook executables
- FastHooks expects tools to be already installed
- You may need to install dependencies manually

## From cargo-husky (Rust)

### Configuration Comparison

**Cargo.toml**
```toml
[dev-dependencies]
cargo-husky = { version = "1", features = ["precommit-hook", "run-cargo-fmt"] }
```

**fasthooks.toml**
```toml
version = "1"

[hooks.pre-commit]
[[hooks.pre-commit.tasks]]
name = "cargo fmt"
run = "cargo fmt --check"

[[hooks.pre-commit.tasks]]
name = "cargo clippy"
run = "cargo clippy -- -D warnings"

[hooks.pre-push]
[[hooks.pre-push.tasks]]
name = "cargo test"
run = "cargo test"
```

## Verifying Migration

After migrating, verify your setup:

```bash
# List configured hooks
fasthooks list

# Test pre-commit hook
fasthooks run pre-commit

# Run benchmark to compare
fasthooks benchmark
```

## Troubleshooting

### Commands Not Found

Ensure all tools are in your PATH:

```bash
# Check if a tool is available
which eslint
which prettier
```

### Different Working Directory

If your tools need to run from a specific directory:

```toml
[[hooks.pre-commit.tasks]]
name = "frontend lint"
run = "npm run lint"
cwd = "frontend"
```

### Environment Variables

Some tools require specific environment variables:

```toml
[[hooks.pre-commit.tasks]]
name = "test"
run = "npm test"
[hooks.pre-commit.tasks.env]
NODE_ENV = "test"
CI = "true"
```
