# 你好，世界
This is sample text

# Second Header
hello

```rust
use clap::{Parser, Subcommand};
use guard::agent::{AgentRunner, AgentType};
use guard::config::{ConfigParser, GuardConfig};
use guard::git::GitOperations;
use guard::output::Reporter;
use guard::watcher::{EventHandler, FileWatcher};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "guard")]
#[command(about = "Protect files from coding agent modifications", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```
