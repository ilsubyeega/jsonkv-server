use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::{
    config::{Config, Secrets},
    service::KeyService,
};

pub struct AppContext {
    pub config: Config,
    pub secrets: Arc<RwLock<Secrets>>,

    pub hashmap: Arc<RwLock<std::collections::HashMap<String, serde_json::Value>>>,
    pub sender_file_save: mpsc::Sender<(String, serde_json::Value)>,
    pub broadcast: broadcast::Sender<(String, serde_json::Value)>,

    pub key_service: Arc<KeyService>,
}
