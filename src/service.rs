pub struct KeyService;

pub trait KeyServiceTrait {
    async fn get_key(&self, key: String) 
        -> Result<serde_json::Value, Box<dyn std::error::Error>>;
    async fn post_key(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn put_key(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn patch_key(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn list_keys(&self) 
        -> Result<Vec<String>, Box<dyn std::error::Error>>;
}

impl KeyServiceTrait for KeyService {
    async fn get_key(&self, key: String) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        Ok(serde_json::Value::Null)
    }

    async fn post_key(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn put_key(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn patch_key(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn list_keys(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }
}
