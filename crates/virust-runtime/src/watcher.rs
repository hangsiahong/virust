use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::sync::mpsc;

pub fn create_watcher(
    path: &Path,
    tx: mpsc::Sender<()>,
) -> Result<RecommendedWatcher, notify::Error> {
    let mut watcher = notify::recommended_watcher(move |_| {
        let _ = tx.blocking_send(());
    })?;

    watcher.watch(path, RecursiveMode::Recursive)?;
    Ok(watcher)
}