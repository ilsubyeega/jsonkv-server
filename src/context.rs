use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::config::{Config, Secrets};

pub struct AppContext {
    pub config: Config,
    pub secrets: Arc<RwLock<Secrets>>,

    pub hashmap: Arc<RwLock<std::collections::HashMap<String, serde_json::Value>>>,
    pub sender_filesave: mpsc::Sender<(String, serde_json::Value)>,

    pub keyservice: Arc<crate::service::KeyService>,
}
