use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, Path, Request, State, WebSocketUpgrade,
    },
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_extra::{
    headers::{self},
    TypedHeader,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{config::Secrets, context::AppContext};
pub async fn create_router(context: Arc<AppContext>) -> Router {
    Router::new()
        .route("/", get(index))
        .merge(
            Router::new()
                .route(
                    "/data/:key",
                    get(get_key).post(post_key).put(put_key).patch(patch_key),
                )
                .route("/listen/:key", get(ws_key))
                .route("/list", get(list_keys))
                .route_layer(middleware::from_fn_with_state(context.clone(), auth_layer))
                .with_state(context),
        )
        .fallback(handle_404)
}

async fn auth_layer(
    State(context): State<Arc<AppContext>>,
    request: Request,
    next: Next,
) -> Response {
    if let Some(auth) = request.headers().get("Authorization") {
        if check_auth(auth, &context.secrets).await {
            return next.run(request).await;
        }
    }

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from("Unauthorized"))
        .unwrap()
}

async fn check_auth(auth: &HeaderValue, secrets: &Arc<Mutex<Secrets>>) -> bool {
    let authstr = auth.to_str();
    if authstr.is_err() {
        return false;
    }
    let auth = authstr.unwrap();
    // Trim the leading "Bearer " from the auth string.
    let auth = auth.trim_start_matches("Bearer ");
    let secrets = secrets.lock().await;
    secrets.contains_key(auth)
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
