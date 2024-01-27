use std::net::SocketAddr;

use axum::{
    extract::{ws::{Message, WebSocket}, ConnectInfo, Path, WebSocketUpgrade},
    handler,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_extra::{
    headers::{self},
    TypedHeader,
};

pub async fn create_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route(
            "/data/:key",
            get(get_key).post(post_key).put(put_key).patch(patch_key),
        )
        .route(
            "/listen/:key",
            get(ws_key)
        )
        .route(
            "/list",
            get(list_keys)
        )
        .fallback(handle_404)
}

async fn index() -> impl IntoResponse {
    String::from("jsonkv-server: https://github.com/ilsubyeega/jsonkv-server")
}

async fn handle_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 Not Found")
}

async fn get_key(Path(key): Path<String>) -> impl IntoResponse {
    (StatusCode::OK, format!("GET OK {}", key))
}

async fn post_key(Path(key): Path<String>) -> impl IntoResponse {
    (StatusCode::OK, format!("POST OK {}", key))
}

async fn put_key(Path(key): Path<String>) -> impl IntoResponse {
    (StatusCode::OK, format!("PUT OK {}", key))
}

async fn patch_key(Path(key): Path<String>) -> impl IntoResponse {
    (StatusCode::OK, format!("PATCH OK {}", key))
}

async fn list_keys() -> impl IntoResponse {
    (StatusCode::OK, "LIST OK")
}

async fn ws_key(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(key): Path<String>
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };

    println!("WS {key}: `{user_agent}` at {addr} connected.");
    ws.on_upgrade(move |socket| handle_websocket(socket, key, addr))
}

async fn handle_websocket(mut socket: WebSocket, key: String, who: SocketAddr) {
    if !socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        println!("Could not send ping {who}!");
        return;
    }

    todo!()
}
