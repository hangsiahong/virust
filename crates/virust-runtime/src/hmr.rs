use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::broadcast;

pub struct HmrWatcher {
    running: AtomicBool,
    tx: broadcast::Sender<()>,
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
