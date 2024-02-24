use std::{io::Read, path::Path, sync::Arc};

use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::service::{KeyService, KeyServiceTrait};
/// File read worker
/// This worker reads from file and compares the data, then modify if is modified.
pub async fn file_read_worker(
    data_dir_path: &str,
    mut rx: Receiver<String>,
    key_service: Arc<KeyService>,
) {
    let path = Path::new(data_dir_path);
    loop {
        let key = rx.recv().await.unwrap();

        // append the data_dir_path to the path.
        let file_path = path.join(format!("{key}.json"));

        // check the file exist
        if !file_path.is_file() {
            continue;
        }

        // read the file
        let file = std::fs::File::open(file_path);
        if file.is_err() {
            println!("failed to open file: {:?}", file.err());
            continue;
        }

        // try to get the key from key_service.
        let res = key_service.get_key(&key).await;
        let is_key_exists = match res {
            Ok(_) => true,
            Err(_) => false,
        };

        let file = file.unwrap();

        // if string is empty, mark this as null.
        let mut reader = std::io::BufReader::new(file);
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();

        if text.is_empty() {
            if is_key_exists {
                key_service
                    .put_key(&key, serde_json::Value::Null)
                    .await
                    .unwrap();
            } else {
                key_service
                    .post_key(&key, serde_json::Value::Null)
                    .await
                    .unwrap();
            }

            continue;
        }

        // parse the file
        let parsed: serde_json::Value = match serde_json::from_str(&text) {
            Ok(parsed) => parsed,
            Err(e) => {
                println!("failed to parse file: {:?}", e);
                continue;
            }
        };

        // if key exists, compare the value.
        if is_key_exists {
            let res = key_service.get_key(&key).await;
            let value = match res {
                Ok(value) => value,
                Err(_) => {
                    println!("failed to get key from key_service");
                    continue;
                }
            };

            if value != parsed {
                key_service.put_key(&key, parsed).await.unwrap();
            }
        } else {
            key_service.post_key(&key, parsed).await.unwrap();
        }
    }
}
