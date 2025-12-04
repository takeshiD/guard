# guard - File protection tool for coding agents

`guard` is a CLI tool that protects specific line ranges in files from being
modified by coding agents such as Claude Code, OpenAI Codex, and Gemini CLI.

- Define protected files and line ranges in `.guard.toml`
- Watch the filesystem for changes
- Automatically rollback protected ranges via Git when a violation is detected

---

## Install / Build

From the repository root:

```bash
# Development: run `guard` without installing
cargo run -- <subcommand> [args...]

# Install locally
cargo install --path .

# Verify installation
guard --help
```

Requirements:

- Rust 1.70 or later
- Git 2.30 or later

---

## Basic usage

### 1. Initialize a Git project

Ensure your target project is under Git:

```bash
git init
git add .
git commit -m "chore: initial commit"
```

### 2. Initialize `.guard.toml`

From the project root you want to protect:

```bash
guard init
```

This creates `.guard.toml` in the current directory.
If it already exists, it is left untouched and a warning is printed.

### 3. Edit `.guard.toml` to define protected ranges

Open the generated `.guard.toml` and define which files and ranges to protect:

```toml
version = "1.0"

[[guards]]
path = "src/main.rs"
reason = "Critical authentication logic"
ranges = [
  { start = 10, end = 50 },
]

[[guards]]
path = "src/lib.rs"
reason = "Public API contract"
ranges = [
  { start = 1, end = 20 },
]

[settings]
auto_rollback = true          # automatically run `git restore` on violation
log_file = ".guard/guard.log" # log output path
strict_mode = false           # if true, guard exits with error on violation
```

Notes:

- `path` is relative to the repository root
- Line numbers are 1-based
- `ranges` for the same file must not overlap

### 4. Run your agent through guard

Start an agent with protection enabled (example: codex):

```bash
# run codex via guard
guard codex -- <codex用の引数...>

# 例
guard codex -- --config codex.toml
```

For Claude Code / Gemini CLI:

```bash
guard claude-code -- <args>
guard gemini-cli -- <args>
```

How it works:

1. Load and validate `.guard.toml`
2. Take a snapshot of the current Git state
3. Start watching directories containing protected files
4. Spawn the agent command (`codex` / `claude-code` / `gemini-cli`)
5. On each file change event, compare against Git HEAD line by line
6. If a change falls inside a protected range:
   - Print a warning
   - Append a log entry to `.guard/guard.log`
   - If `auto_rollback = true`, restore the file via `git restore`
   - If `strict_mode = true`, exit guard with an error

The agent runs as a child process of guard, and guard keeps watching until the
agent exits.

### 5. Validate configuration only

You can validate `.guard.toml` without starting an agent:

```bash
guard check
# または
guard check --config path/to/.guard.toml
```

Guard will check:

- Whether `version` is supported (`"1.0"`)
- That each `path` exists
- That line ranges are 1-based and `start <= end`
- That ranges for the same file do not overlap

---

## Typical workflow

```bash
# 1. Initialize guard at the project root
guard init

# 2. Edit .guard.toml to define protected ranges
$EDITOR .guard.toml

# 3. Run your agent via guard
guard codex -- --config codex.toml

# 4. If the agent modifies a protected range,
#    guard detects the violation, rolls back, and logs it

# 5. Inspect the log
cat .guard/guard.log
```

---

## Limitations and caveats

- The target project must be a Git repository.
- Rollback uses `git restore`, which resets files to the state in HEAD,
  including uncommitted changes.
- The agent commands (`codex`, `claude-code`, `gemini-cli`, etc.) must be
  available on your `PATH`.

---

## For contributors

### Tests / static analysis

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

### Main modules

- `src/config/` : `.guard.toml` parsing and validation
- `src/watcher/` : filesystem watching and event handling
- `src/detector/` : detection of modifications inside protected ranges
- `src/git/` : Git snapshots and file restore
- `src/agent/` : coding agent process management
- `src/output/` : logging and session reports
