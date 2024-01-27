use std::sync::Arc;

use dotenvy::dotenv;
use tokio::{net::TcpListener, sync::Mutex};
mod config;
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

    let data = filesave::load_data_from_disk(&config.data_dir_path).await.unwrap();

    let shared_data = Arc::new(Mutex::new(data));

    let router = server::create_router().await;
    let listener = match listen {
        config::ListenType::Http(addr) => TcpListener::bind(addr).await.unwrap(),
        config::ListenType::Unix(_) => todo!() // tricky task
    };

    axum::serve(listener, router.into_make_service()).await.unwrap();
}