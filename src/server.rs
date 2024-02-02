use std::net::SocketAddr;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, Path, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::{
    headers::{self},
    TypedHeader,
};
use std::sync::Arc;

use crate::context::AppContext;
pub async fn create_router(context: Arc<AppContext>) -> Router {
    Router::new()
        .route("/", get(index))
        .route(
            "/data/:key",
            get(get_key).post(post_key).put(put_key).patch(patch_key),
        )
        .route("/listen/:key", get(ws_key))
        .route("/list", get(list_keys))
        .with_state(context)
        .fallback(handle_404)
}

async fn index() -> impl IntoResponse {
    String::from("jsonkv-server: https://github.com/ilsubyeega/jsonkv-server")
}

async fn handle_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 Not Found")
}

async fn get_key(
    State(context): State<Arc<AppContext>>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    (StatusCode::OK, format!("GET OK {}", key))
}

async fn post_key(
    State(context): State<Arc<AppContext>>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    (StatusCode::OK, format!("POST OK {}", key))
}

async fn put_key(
    State(context): State<Arc<AppContext>>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    (StatusCode::OK, format!("PUT OK {}", key))
}

async fn patch_key(
    State(context): State<Arc<AppContext>>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    (StatusCode::OK, format!("PATCH OK {}", key))
}

async fn list_keys(State(context): State<Arc<AppContext>>) -> impl IntoResponse {
    (StatusCode::OK, "LIST OK")
}

async fn ws_key(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(context): State<Arc<AppContext>>,
    Path(key): Path<String>,
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
