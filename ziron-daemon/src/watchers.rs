//! File system and Git repository watchers

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use tokio::sync::broadcast;
use ziron_core::cache::Cache;
use ziron_core::event::Event as ZironEvent;

/// Watcher manager for file system and Git changes
pub struct WatcherManager {
    file_watcher: Option<RecommendedWatcher>,
    cache: Cache,
    event_tx: broadcast::Sender<ZironEvent>,
}

impl WatcherManager {
    /// Create a new watcher manager
    pub fn new(cache: Cache, event_tx: broadcast::Sender<ZironEvent>) -> Self {
        Self {
            file_watcher: None,
            cache,
            event_tx,
        }
    }

    /// Start watching a directory
    pub fn watch_directory(&mut self, path: &Path) -> Result<(), notify::Error> {
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(path, RecursiveMode::Recursive)?;
        
        self.file_watcher = Some(watcher);
        
        // Spawn task to handle file system events
        let cache_clone = self.cache.clone();
        let event_tx_clone = self.event_tx.clone();
        tokio::spawn(async move {
            Self::handle_file_events(rx, cache_clone, event_tx_clone).await;
        });
        
        Ok(())
    }

    /// Handle file system events
    async fn handle_file_events(
        rx: mpsc::Receiver<Result<Event, notify::Error>>,
        cache: Cache,
        event_tx: broadcast::Sender<ZironEvent>,
    ) {
        while let Ok(event_result) = rx.recv() {
            match event_result {
                Ok(event) => {
                    // Invalidate cache for affected paths
                    // For now, invalidate all cache entries when any file changes
                    // TODO: Implement more granular cache invalidation
                    cache.invalidate(None);
                    
                    // Emit directory change event
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)) {
                        if let Some(path) = event.paths.first() {
                            if let Some(path_str) = path.to_str() {
                                let _ = event_tx.send(ZironEvent::directory_change(path_str.to_string()));
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("File watcher error: {}", e);
                }
            }
        }
    }

    /// Check if a directory is a Git repository
    pub fn is_git_repo(path: &Path) -> bool {
        path.join(".git").exists()
    }

    /// Watch Git repository for changes
    pub fn watch_git_repo(&mut self, repo_path: &Path) -> Result<(), notify::Error> {
        if !Self::is_git_repo(repo_path) {
            return Err(notify::Error::generic("Not a Git repository"));
        }
        
        // Watch .git directory for changes
        let git_path = repo_path.join(".git");
        self.watch_directory(&git_path)
    }
}

