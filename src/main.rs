use dotenvy::dotenv;
use std::sync::Arc;
use std::{future::IntoFuture, hash};
use tokio::{
    net::TcpListener,
    sync::{mpsc, RwLock},
};
mod config;
mod context;
mod server;
mod service;
mod websocket;
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

    let data = file_save::load_data_from_disk(&config.data_dir_path)
        .await
        .unwrap();

    let file_save = mpsc::channel(1000);
    let file_listen = mpsc::channel(32);
    let broadcaster = mpsc::channel(32);

    let broadcast = tokio::sync::broadcast::channel(32);

    let hashmap = Arc::new(RwLock::new(data));

    let context = Arc::new(context::AppContext {
        config: config.clone(),
        secrets: Arc::new(RwLock::new(secrets)),
        hashmap: hashmap.clone(),
        sender_file_save: file_save.0.clone(),
        broadcast: broadcast.0.clone(),

        key_service: Arc::new(service::KeyService {
            hashmap: hashmap.clone(),
            sender_file_save: file_save.0.clone(),
            broadcaster: broadcaster.0,
        }),
    });

    let router = server::create_router(context.clone()).await;
    let listener = match listen.clone() {
        config::ListenType::Http(addr) => TcpListener::bind(addr).await.unwrap(),
        config::ListenType::Unix(_) => todo!(), // tricky task
    };

    println!("Listening on: {:?}", listen);
    tokio::select! {
        _ = axum::serve(listener, router).into_future() => (),
        _ = workers::file_save::save_data_worker(file_save.1, config.data_dir_path.clone(), config.save_interval) => (),
        _ = workers::file_listen::file_listen_worker(&config.data_dir_path, file_listen.0) => (),
        _ = workers::file_read::file_read_worker(&config.data_dir_path, file_listen.1, context.key_service.clone()) => (),
        _ = workers::broadcaster::worker_broadcaster(broadcaster.1, broadcast.0) => (),
    }
}
