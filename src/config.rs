use serde::{Deserialize, Serialize};
use std::{env, fs};

#[derive(Debug, Clone)]
pub struct Config {
    /// The address to listen on.
    pub listen: String,
    /// The data directory, where every datas are saved.
    pub data_dir_path: String,
    /// The path to the secret file.
    pub secret_file_path: String,
    /// Enable `GET /list` route.
    pub enable_list: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen: "127.0.0.1:19720".to_owned(),
            data_dir_path: "./data".to_owned(),
            secret_file_path: "./secret.toml".to_owned(),
            enable_list: true,
        }
    }
}

/// Parse the config from environment variables.
pub fn parse_from_env() -> Config {
    let mut config = Config::default();
    if let Ok(listen) = env::var("JSONKV_LISTEN") {
        config.listen = listen;
    }
    if let Ok(data_dir_path) = env::var("JSONKV_DATA_DIR") {
        config.data_dir_path = data_dir_path;
    }
    if let Ok(secret_file_path) = env::var("JSONKV_SECRET_FILE") {
        config.secret_file_path = secret_file_path;
    }
    if let Ok(enable_list) = env::var("JSONKV_ENABLE_LIST") {
        config.enable_list = enable_list.parse().unwrap();
    }
    config
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub secret: String,
    pub name: String,
    pub description: Option<String>,
}

/// The wrapper of `Secret`. \
/// See https://stackoverflow.com/a/74090664 for why we need this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secrets {
    pub secret: Vec<Secret>,
}

/// Load the secrets from the given path.
/// If the file does not exist, create a sample and save it to the given path
pub fn load_secrets(path: &str) -> Secrets {
    let path = std::path::Path::new(path);
    if path.exists() {
        let content = std::fs::read_to_string(path).unwrap();
        let secrets: Secrets = toml::from_str(&content).unwrap();
        secrets
    } else {
        let secret_sample = Secret {
            secret: "secret".to_owned(),
            name: "name".to_owned(),
            description: None,
        };
        let another_secret = Secret {
            secret: "another secret".to_owned(),
            name: "another name".to_owned(),
            ..secret_sample.clone()
        };

        let secrets = Secrets {
            secret: vec![secret_sample, another_secret],
        };

        let serialized = toml::to_string(&secrets).unwrap();
        fs::write(path, serialized).unwrap();
        secrets
    }
}

#[derive(Debug, Clone)]
pub enum ListenType {
    Http(String),
    Unix(String),
}
/// Parse the listen address from the given string.
/// It could be `127.0.0.1:3000` or `unix:/tmp/jsonkv.sock`.
pub fn parse_listen(listen: &str) -> ListenType {
    if listen.starts_with("unix:") {
        let path = listen[5..].to_owned();
        ListenType::Unix(path)
    } else {
        // check this is valid address.
        let _: std::net::SocketAddr = listen.parse().unwrap();
        ListenType::Http(listen.to_owned())
    }
}