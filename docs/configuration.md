# Configuration Reference

FastHooks uses a TOML configuration file (`fasthooks.toml`) in your project root.

## File Location

FastHooks looks for configuration in the following order:

1. `fasthooks.toml`
2. `.fasthooks.toml`
3. `fasthooks.yaml`
4. `.fasthooks.yaml`

## Basic Structure

```toml
version = "1"

[settings]
# Global settings

[hooks.pre-commit]
# Hook configuration

[[hooks.pre-commit.tasks]]
# Task definitions
```

## Settings

### Global Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `parallel` | bool | `true` | Run tasks in parallel |
| `max_parallel` | int | `0` | Max parallel tasks (0 = auto-detect CPU cores) |
| `show_stats` | bool | `true` | Show execution statistics |
| `show_carbon_savings` | bool | `true` | Show estimated carbon savings |
| `fail_fast` | bool | `true` | Stop on first error |
| `skip_ci` | bool | `false` | Skip hooks in CI environment |
| `colors` | bool | `true` | Enable colored output |

### Example

```toml
[settings]
parallel = true
max_parallel = 4
show_stats = true
show_carbon_savings = true
fail_fast = true
skip_ci = false
colors = true
```

## Hooks

FastHooks supports all standard Git hooks:

| Hook | When it runs |
|------|--------------|
| `pre-commit` | Before a commit is created |
| `prepare-commit-msg` | Before the commit message editor opens |
| `commit-msg` | After commit message is entered |
| `post-commit` | After a commit is created |
| `pre-push` | Before pushing to remote |
| `pre-rebase` | Before a rebase starts |
| `post-checkout` | After switching branches |
| `post-merge` | After a merge completes |
| `pre-auto-gc` | Before auto garbage collection |

### Hook Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `parallel` | bool | inherit | Override global parallel setting |
| `fail_fast` | bool | inherit | Override global fail_fast setting |
| `skip_ci` | bool | inherit | Override global skip_ci setting |

### Example

```toml
[hooks.pre-commit]
parallel = true
fail_fast = true

[hooks.pre-push]
parallel = false  # Run sequentially
```

## Tasks

Tasks are the individual commands that run within a hook.

### Task Options

| Option | Type | Required | Default | Description |
|--------|------|----------|---------|-------------|
| `name` | string | yes | - | Display name for the task |
| `run` | string | yes | - | Command to execute |
| `glob` | string | no | - | Glob pattern for file matching |
| `staged` | bool | no | `true` | Only run on staged files |
| `cwd` | string | no | `.` | Working directory |
| `env` | table | no | `{}` | Environment variables |
| `allow_failure` | bool | no | `false` | Continue if task fails |
| `if` | string | no | - | Condition for running |

### Basic Task

```toml
[[hooks.pre-commit.tasks]]
name = "lint"
run = "npm run lint"
```

### Task with Glob Pattern

```toml
[[hooks.pre-commit.tasks]]
name = "format typescript"
run = "prettier --write"
glob = "*.{ts,tsx}"
staged = true
```

The `{files}` placeholder will be replaced with matching files:

```toml
[[hooks.pre-commit.tasks]]
name = "eslint"
run = "eslint --fix {files}"
glob = "*.{js,ts}"
```

### Task with Environment Variables

```toml
[[hooks.pre-commit.tasks]]
name = "test"
run = "npm test"
[hooks.pre-commit.tasks.env]
NODE_ENV = "test"
CI = "true"
```

### Task with Working Directory

```toml
[[hooks.pre-commit.tasks]]
name = "backend tests"
run = "cargo test"
cwd = "backend"
```

### Conditional Task

```toml
[[hooks.pre-commit.tasks]]
name = "e2e tests"
run = "npm run test:e2e"
if = "branch == main"

[[hooks.pre-commit.tasks]]
name = "quick tests"
run = "npm test"
if = "branch != main"
```

### Allow Failure

```toml
[[hooks.pre-commit.tasks]]
name = "optional check"
run = "npm run lint:experimental"
allow_failure = true
```

## Glob Patterns

FastHooks uses standard glob patterns compatible with lint-staged:

| Pattern | Matches |
|---------|---------|
| `*.js` | All JavaScript files |
| `*.{js,ts}` | JavaScript and TypeScript files |
| `src/**/*.ts` | TypeScript files in src directory |
| `!*.test.js` | Exclude test files |
| `**/*.{css,scss}` | CSS and SCSS files anywhere |

### Examples

```toml
# JavaScript/TypeScript
[[hooks.pre-commit.tasks]]
name = "eslint"
run = "eslint --fix"
glob = "*.{js,jsx,ts,tsx}"

# Styles
[[hooks.pre-commit.tasks]]
name = "stylelint"
run = "stylelint --fix"
glob = "*.{css,scss,less}"

# Documentation
[[hooks.pre-commit.tasks]]
name = "markdownlint"
run = "markdownlint"
glob = "*.md"

# Specific directory
[[hooks.pre-commit.tasks]]
name = "backend lint"
run = "cargo clippy"
glob = "backend/**/*.rs"
```

## Complete Example

```toml
# FastHooks Configuration
version = "1"

[settings]
parallel = true
max_parallel = 0
show_stats = true
show_carbon_savings = true
fail_fast = true
skip_ci = false
colors = true

# Pre-commit hooks
[hooks.pre-commit]
parallel = true

[[hooks.pre-commit.tasks]]
name = "eslint"
run = "eslint --fix"
glob = "*.{js,ts,jsx,tsx}"
staged = true

[[hooks.pre-commit.tasks]]
name = "prettier"
run = "prettier --write"
glob = "*.{js,ts,jsx,tsx,json,md,css}"
staged = true

[[hooks.pre-commit.tasks]]
name = "typecheck"
run = "tsc --noEmit"

[[hooks.pre-commit.tasks]]
name = "stylelint"
run = "stylelint --fix"
glob = "*.{css,scss}"
staged = true

# Commit message validation
[hooks.commit-msg]
[[hooks.commit-msg.tasks]]
name = "commitlint"
run = "npx commitlint --edit $1"

# Pre-push hooks
[hooks.pre-push]
parallel = false

[[hooks.pre-push.tasks]]
name = "test"
run = "npm test"

[[hooks.pre-push.tasks]]
name = "build"
run = "npm run build"
```

## Environment Detection

FastHooks automatically detects CI environments by checking these variables:

- `CI`
- `CONTINUOUS_INTEGRATION`
- `GITHUB_ACTIONS`
- `GITLAB_CI`
- `CIRCLECI`
- `TRAVIS`
- `JENKINS_URL`
- `BUILDKITE`
- `DRONE`
- `AZURE_PIPELINES`
- `TEAMCITY_VERSION`

Use `skip_ci = true` to skip hooks in CI environments.
