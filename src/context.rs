use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub struct AppContext {
    pub hashmap: Arc<Mutex<std::collections::HashMap<String, serde_json::Value>>>,
    pub sender_filesave: mpsc::Sender<(String, serde_json::Value)>,
}