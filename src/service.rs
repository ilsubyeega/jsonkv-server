use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub struct KeyService {
    // cloned from appcontext.
    pub hashmap: Arc<RwLock<std::collections::HashMap<String, serde_json::Value>>>,
    pub sender_filesave: mpsc::Sender<(String, serde_json::Value)>,
}

pub trait KeyServiceTrait {
    /// Get a key from the hashmap
    async fn get_key(&self, key: String) -> Result<serde_json::Value, KeyServiceError>;
    /// Post a key to the hashmap
    async fn post_key(&self, key: String, value: serde_json::Value) -> Result<(), KeyServiceError>;
    /// Put a key to the hashmap
    /// It's same as `post_key`
    async fn put_key(&self, key: String, value: serde_json::Value) -> Result<(), KeyServiceError>;
    /// Patch a key to the hashmap
    /// It uses RFC-6902 for modifying the value.
    async fn patch_key(&self, key: String, value: serde_json::Value)
        -> Result<(), KeyServiceError>;
    async fn list_keys(&self) -> Result<Vec<String>, KeyServiceError>;
}

impl KeyServiceTrait for KeyService {
    async fn get_key(&self, key: String) -> Result<serde_json::Value, KeyServiceError> {
        {
            let hashmap = self.hashmap.read().await;
            if let Some(value) = hashmap.get(&key) {
                return Ok(value.clone());
            }
        }
        Ok(serde_json::Value::Null)
    }

    async fn post_key(&self, key: String, value: serde_json::Value) -> Result<(), KeyServiceError> {
        {
            let mut hashmap = self.hashmap.write().await;
            hashmap.insert(key.clone(), value.clone());
        }
        // Sends to the filesave channel in order to save the data to the file.
        self.sender_filesave
            .send((key.clone(), value.clone()))
            .await
            .unwrap();
        Ok(())
    }

    async fn put_key(&self, key: String, value: serde_json::Value) -> Result<(), KeyServiceError> {
        self.post_key(key, value).await
    }

    async fn patch_key(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), KeyServiceError> {
        // Parse the json-patch on value parameter first.
        let patch_data: json_patch::Patch = serde_json::from_value(value)
            .map_err(|err| KeyServiceError::UnableToParsePatch(err))?;
        let mut data = {
            let hashmap = self.hashmap.write().await;
            hashmap
                .get(&key)
                .ok_or(KeyServiceError::KeyNotFound)?
                .clone()
        };
        json_patch::patch(&mut data, &patch_data)
            .map_err(|err| KeyServiceError::UnableToPatch(err))?;
        self.put_key(key, data).await
    }

    async fn list_keys(&self) -> Result<Vec<String>, KeyServiceError> {
        let list = {
            let hashmap = self.hashmap.read().await;
            hashmap.keys().cloned().collect()
        };
        Ok(list)
    }
}

#[derive(Debug)]
pub enum KeyServiceError {
    KeyNotFound,
    UnableToParsePatch(serde_json::Error),
    UnableToPatch(json_patch::PatchError),
}

impl std::fmt::Display for KeyServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyServiceError::KeyNotFound => write!(f, "Key not found"),
            KeyServiceError::UnableToParsePatch(err) => {
                write!(f, "Unable to parse the patch: {}", err)
            }
            KeyServiceError::UnableToPatch(err) => write!(f, "Unable to patch: {}", err),
        }
    }
}
