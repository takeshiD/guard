//! ファイルシステム監視ラッパー

use anyhow::Result;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<Event>,
}

impl FileWatcher {
    /// 新しいFileWatcherを作成
    pub fn new(watch_dirs: Vec<PathBuf>) -> Result<Self> {
        let (tx, rx): (Sender<Event>, Receiver<Event>) = channel();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    // Modifyイベントのみ処理
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

    /// イベントを待機（非ブロッキング）
    pub fn try_recv(&self) -> Option<Event> {
        self.receiver.try_recv().ok()
    }

    /// イベントを待機（ブロッキング、タイムアウト付き）
    pub fn recv_timeout(&self, timeout: std::time::Duration) -> Option<Event> {
        self.receiver.recv_timeout(timeout).ok()
    }
}

