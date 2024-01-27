use std::collections::HashMap;

use tokio::sync::mpsc;

/// The changer worker is responsible for changing the data on memory.
/// It receives the data from the server worker and broadcast the data to the other workers.
pub async fn changer_worker(
    mut data_events: mpsc::Receiver<(String, serde_json::Value)>, // K, V.
    mut data: HashMap<String, serde_json::Value>,
    data_events_broadcast: mpsc::Sender<(String, serde_json::Value)>,
) {
    loop {
        tokio::select! {
            Some(data_event) = data_events.recv() => {
                // if key exists, update the value.
                // if key does not exist, insert the key-value pair.
                let cloned = data_event.clone();
                data.insert(cloned.0, cloned.1);
                data_events_broadcast.send(data_event).await.unwrap();
            }
        }
    }
}