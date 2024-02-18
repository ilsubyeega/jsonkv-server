use std::collections::HashMap;
use tokio::sync::mpsc;
/// This collects data events about each modified piece of data as it comes in, and stores the latest data every n seconds.
///
/// # Arguments
///
/// * `data_events` - The receiver of data events.
/// * `data_dir_path` - The path to the data directory.
/// * `save_interval` - The interval to save the data to disk. (in milliseconds)
pub async fn save_data_worker(
    mut data_events: mpsc::Receiver<(String, serde_json::Value)>, // K, V.
    data_dir_path: String,
    save_interval: u64,
) {
    let mut data = HashMap::new();
    loop {
        tokio::select! {
            Some(data_event) = data_events.recv() => {
                // if key exists, update the value.
                // if key does not exist, insert the key-value pair.
                data.insert(data_event.0, data_event.1);
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(save_interval)) => {
                if let Err(e) = save_data_to_disk(&data, &data_dir_path).await {
                    panic!("failed to save data to disk: {}", e);
                }

                data.clear();
            }
        }
    }
}

/// Save the given data to the given path.
/// If the file does not exist, create a sample and save it to the given path.
/// Single key is just `[key].json`
/// If the file exists, overwrite it.
async fn save_data_to_disk(
    data: &HashMap<String, serde_json::Value>,
    data_dir_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // data dir should be exist at this moment.
    let data_dir_path = std::path::Path::new(data_dir_path);
    if !data_dir_path.exists() {
        return Err("data dir does not exist".into());
    }

    for (key, value) in data {
        let file_path = data_dir_path.join(format!("{}.json", key));
        let file_path = file_path.to_str().unwrap();
        let file = std::fs::File::create(file_path)?;
        serde_json::to_writer_pretty(file, value)?;
    }

    Ok(())
}

pub async fn load_data_from_disk(
    data_dir_path: &str,
) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error>> {
    let data_dir_path = std::path::Path::new(data_dir_path);
    if !data_dir_path.exists() {
        return Err("data dir does not exist".into());
    }

    let mut data = HashMap::new();
    for entry in std::fs::read_dir(data_dir_path)? {
        // check if the file extension is json.
        let entry = entry?;
        let file_path = entry.path();
        let file_path = file_path.to_str().unwrap();
        if !file_path.ends_with(".json") {
            continue;
        }

        // read the file and insert to the data.
        let file = std::fs::File::open(file_path)?;
        let key = file_path
            .split('/')
            .last()
            .unwrap()
            .split('.')
            .next()
            .unwrap(); // remove the extension.

        // if file is empty, skip it.
        let value = if file.metadata()?.len() == 0 {
            serde_json::Value::Null
        } else {
            serde_json::from_reader(file).unwrap_or_else(|_| {
                format!("failed to parse file: {}", file_path);
                serde_json::Value::Null
            })
        };
        data.insert(key.to_owned(), value);
    }

    Ok(data)
}
