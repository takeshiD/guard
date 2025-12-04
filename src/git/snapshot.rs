//! Gitスナップショットの型

#[derive(Debug, Clone)]
pub struct GitSnapshot {
    pub branch: String,
    pub head_hash: String,
    pub has_uncommitted: bool,
}

