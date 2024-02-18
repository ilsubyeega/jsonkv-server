use std::sync::mpsc;

use tokio::sync::{broadcast, mpsc::Receiver};

/// Broadcaster worker
/// This worker broadcasts the changed data.
pub async fn worker_broadcaster(
    mut rx: Receiver<(String, serde_json::Value)>,
    tx: broadcast::Sender<(String, serde_json::Value)>,
) {
    loop {
        let data = rx.recv().await.unwrap();
        tx.send(data).unwrap();
    }
}
