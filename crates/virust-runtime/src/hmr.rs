use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::broadcast;

pub struct HmrWatcher {
    running: AtomicBool,
    tx: broadcast::Sender<()>,
}

// Safety: HmrWatcher only contains AtomicBool and broadcast::Sender,
// which are both Send + Sync
unsafe impl Send for HmrWatcher {}
unsafe impl Sync for HmrWatcher {}

impl Clone for HmrWatcher {
    fn clone(&self) -> Self {
        Self {
            running: AtomicBool::new(self.running.load(Ordering::Relaxed)),
            tx: self.tx.clone(),
        }
    }
}

impl HmrWatcher {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);

        Self {
            running: AtomicBool::new(true),
            tx,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn notify_reload(&self) {
        let _ = self.tx.send(());
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hmr_websocket() {
        let watcher = HmrWatcher::new();
        assert!(watcher.is_running());
    }
}
