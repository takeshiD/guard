//! guard - File protection tool for coding agents
//!
//! A Rust CLI tool that prevents coding agents from modifying
//! specific line ranges in selected files.

pub mod config;
pub mod watcher;
pub mod detector;
pub mod git;
pub mod agent;
pub mod output;
pub mod error;

// Re-exports for convenience
pub use error::{GuardError, Result};
