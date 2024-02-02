use dotenvy::dotenv;
use std::sync::Arc;
use std::{future::IntoFuture, hash};
use tokio::{
    net::TcpListener,
    sync::{mpsc, Mutex},
};
mod config;
mod context;
mod server;
mod service;
mod workers;

use crate::workers::*;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() {
    let _ = dotenv();

    let config = config::parse_from_env();
    println!("config: {:?}", config);

    let secrets = config::load_secrets(&config.secret_file_path);
    println!("secrets: {:?}", secrets);

    let listen = config::parse_listen(&config.listen);
    println!("listen: {:?}", listen);

    // Create a data dir if not exists.
    let data_dir_path = std::path::Path::new(&config.data_dir_path);
    if !data_dir_path.exists() {
        std::fs::create_dir_all(data_dir_path).unwrap();
    }

    let data = filesave::load_data_from_disk(&config.data_dir_path)
        .await
        .unwrap();

    let filesave = mpsc::channel(1000);

    let hashmap = Arc::new(Mutex::new(data));

    let context = context::AppContext {
        config: config.clone(),
        secrets: Arc::new(Mutex::new(secrets)),
        hashmap: hashmap.clone(),
        sender_filesave: filesave.0.clone(),

        keyservice: Arc::new(service::KeyService {
            hashmap: hashmap.clone(),
            sender_filesave: filesave.0.clone(),
        }),
    };

    let router = server::create_router(Arc::new(context)).await;
    let listener = match listen.clone() {
        config::ListenType::Http(addr) => TcpListener::bind(addr).await.unwrap(),
        config::ListenType::Unix(_) => todo!(), // tricky task
    };

    println!("Listening on: {:?}", listen);
    tokio::select! {
        _ = axum::serve(listener, router).into_future() => (),
        _ = workers::filesave::save_data_worker(filesave.1, config.data_dir_path, config.save_interval) => ()
    }
}
