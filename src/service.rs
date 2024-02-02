use json_patch::{patch, Patch};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub struct KeyService {
    // cloned from appcontext.
    pub hashmap: Arc<Mutex<std::collections::HashMap<String, serde_json::Value>>>,
    pub sender_filesave: mpsc::Sender<(String, serde_json::Value)>,
}

pub trait KeyServiceTrait {
    /// Get a key from the hashmap
    async fn get_key(&self, key: String) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
    /// Post a key to the hashmap
    async fn post_key(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>>;
    /// Put a key to the hashmap
    /// It's same as `post_key`
    async fn put_key(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>>;
    /// Patch a key to the hashmap
    /// It uses RFC-6902 for modifying the value.
    async fn patch_key(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn list_keys(&self) -> Result<Vec<String>, Box<dyn std::error::Error>>;
}

impl KeyServiceTrait for KeyService {
    async fn get_key(&self, key: String) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        {
            let hashmap = self.hashmap.lock().await;
            if let Some(value) = hashmap.get(&key) {
                return Ok(value.clone());
            }
        }
        Ok(serde_json::Value::Null)
    }

    async fn post_key(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut hashmap = self.hashmap.lock().await;
            hashmap.insert(key.clone(), value.clone());
        }
        // Sends to the filesave channel in order to save the data to the file.
        self.sender_filesave
            .send((key.clone(), value.clone()))
            .await
            .unwrap();
        Ok(())
    }

    async fn put_key(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.post_key(key, value).await
    }

    async fn patch_key(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Parse the json-patch on value parameter first.
        let patch_data: Patch = serde_json::from_value(value)?;
        let mut data = {
            let hashmap = self.hashmap.lock().await;
            hashmap.get(&key).ok_or("Key not found")?.clone()
        };
        patch(&mut data, &patch_data)?;
        self.put_key(key, data).await
    }

    async fn list_keys(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let list = {
            let hashmap = self.hashmap.lock().await;
            hashmap.keys().cloned().collect()
        };
        Ok(list)
    }
}
