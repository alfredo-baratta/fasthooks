# Troubleshooting

Common issues and solutions when using FastHooks.

## Installation Issues

### "fasthooks: command not found"

The FastHooks binary is not in your PATH.

**Solution (Cargo install):**
```bash
# Add cargo bin to PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Or add to your shell profile (.bashrc, .zshrc, etc.)
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
```

**Solution (npm install):**
```bash
# Use npx to run
npx fasthooks --version

# Or add to package.json scripts
{
  "scripts": {
    "prepare": "fasthooks install"
  }
}
```

### "Not a git repository"

FastHooks must be run from within a Git repository.

**Solution:**
```bash
# Initialize a git repository
git init

# Then initialize FastHooks
fasthooks init
```

## Configuration Issues

### "No fasthooks.toml found"

The configuration file doesn't exist or is not in the expected location.

**Solution:**
```bash
# Create configuration
fasthooks init

# Or create manually
touch fasthooks.toml
```

### "Failed to parse TOML configuration"

Your configuration file has syntax errors.

**Common issues:**

1. **Missing quotes around strings with spaces:**
```toml
# Wrong
run = npm run lint

# Correct
run = "npm run lint"
```

2. **Incorrect array syntax:**
```toml
# Wrong
hooks.pre-commit.tasks.name = "lint"

# Correct
[[hooks.pre-commit.tasks]]
name = "lint"
```

3. **Invalid characters:**
```toml
# Wrong (smart quotes)
name = "lint"

# Correct (straight quotes)
name = "lint"
```

### "Unknown hook type"

The hook name is not recognized.

**Valid hook names:**
- `pre-commit`
- `prepare-commit-msg`
- `commit-msg`
- `post-commit`
- `pre-push`
- `pre-rebase`
- `post-checkout`
- `post-merge`
- `pre-auto-gc`

## Execution Issues

### Hook Not Running

The Git hook might not be installed or executable.

**Solution:**
```bash
# Reinstall hooks
fasthooks install

# Check hook file exists and is executable
ls -la .git/hooks/pre-commit
```

### "Permission denied"

The hook script is not executable (Unix systems).

**Solution:**
```bash
chmod +x .git/hooks/pre-commit
```

### Tasks Not Finding Files

The glob pattern might not match any staged files.

**Debug:**
```bash
# List staged files
git diff --cached --name-only

# Test the hook manually
fasthooks run pre-commit
```

**Common glob issues:**

1. **Pattern too restrictive:**
```toml
# Might miss files
glob = "src/*.ts"

# More inclusive
glob = "*.ts"
# Or
glob = "**/*.ts"
```

2. **Staged flag when you want all files:**
```toml
# Only staged files (default)
staged = true

# All matching files
staged = false
```

### Command Fails But Works Manually

The environment might be different when run through FastHooks.

**Solution - Check PATH:**
```toml
[[hooks.pre-commit.tasks]]
name = "lint"
run = "npm run lint"
[hooks.pre-commit.tasks.env]
PATH = "/usr/local/bin:/usr/bin:/bin"
```

**Solution - Use absolute paths:**
```toml
[[hooks.pre-commit.tasks]]
name = "lint"
run = "/usr/local/bin/npm run lint"
```

**Solution - Check working directory:**
```toml
[[hooks.pre-commit.tasks]]
name = "lint"
run = "npm run lint"
cwd = "."  # Explicit working directory
```

### Slow Execution

Tasks are taking longer than expected.

**Solution - Enable parallelization:**
```toml
[settings]
parallel = true
max_parallel = 0  # Auto-detect CPU cores

[hooks.pre-commit]
parallel = true
```

**Solution - Use staged files only:**
```toml
[[hooks.pre-commit.tasks]]
name = "lint"
run = "eslint"
glob = "*.ts"
staged = true  # Only lint changed files
```

## CI/CD Issues

### Hooks Running in CI When They Shouldn't

**Solution:**
```toml
[settings]
skip_ci = true
```

Or for specific hooks:
```toml
[hooks.pre-commit]
skip_ci = true
```

### Hooks Not Running in CI When They Should

FastHooks detects CI environments automatically. If you need hooks to run:

```toml
[settings]
skip_ci = false
```

## Migration Issues

### "No Husky or lint-staged configuration found"

The migration command couldn't find existing configuration.

**Check for:**
- `.husky/` directory
- `lint-staged` in `package.json`
- `.lintstagedrc` or `.lintstagedrc.json`

### Migrated Config Not Working

After migration, review the generated `fasthooks.toml`:

```bash
# Show current configuration
cat fasthooks.toml

# Test hooks
fasthooks run pre-commit
```

## Getting Help

### Debug Output

Enable verbose logging:

```bash
RUST_LOG=debug fasthooks run pre-commit
```

### Check Version

```bash
fasthooks --version
```

### Report Issues

If you encounter a bug:

1. Check existing issues: https://github.com/alfredo-baratta/fasthooks/issues
2. Create a new issue with:
   - FastHooks version
   - Operating system
   - Configuration file
   - Error message
   - Steps to reproduce
