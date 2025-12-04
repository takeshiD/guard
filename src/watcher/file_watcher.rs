//! Thin wrapper around the notify file system watcher

use anyhow::Result;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<Event>,
}

impl FileWatcher {
    /// Create a new `FileWatcher` for the given directories.
    pub fn new(watch_dirs: Vec<PathBuf>) -> Result<Self> {
        let (tx, rx): (Sender<Event>, Receiver<Event>) = channel();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    // Only forward Modify events
                    if matches!(event.kind, notify::EventKind::Modify(_)) {
                        let _ = tx.send(event);
                    }
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        })?;

        for dir in watch_dirs {
            watcher.watch(&dir, RecursiveMode::Recursive)?;
        }

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }

    /// Try to receive an event without blocking.
    pub fn try_recv(&self) -> Option<Event> {
        self.receiver.try_recv().ok()
    }

    /// Wait for an event with a timeout.
    pub fn recv_timeout(&self, timeout: std::time::Duration) -> Option<Event> {
        self.receiver.recv_timeout(timeout).ok()
    }
}
