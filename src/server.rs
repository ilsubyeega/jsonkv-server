use std::net::SocketAddr;

use crate::websocket::handle_websocket;
use axum::{
    body::Body,
    extract::{
        ConnectInfo, Path, Request, State, WebSocketUpgrade,
    },
    http::{HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};
use axum_extra::{
    headers::{self},
    TypedHeader,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{config::Secrets, context::AppContext, service::KeyServiceTrait};
pub async fn create_router(context: Arc<AppContext>) -> Router {
    Router::new()
        .route("/", get(index))
        .merge(
            Router::new()
                .route(
                    "/data/:key",
                    get(get_key).post(post_key).put(put_key).patch(patch_key),
                )
                .route("/list", get(list_keys))
                .route_layer(middleware::from_fn_with_state(context.clone(), auth_layer))
                .layer(CorsLayer::permissive())
                .route("/listen/:key", get(ws_key)) // auth header doesn't work in websocket.
                .with_state(context),
        )
        .fallback(handle_404)
}

async fn auth_layer(
    State(context): State<Arc<AppContext>>,
    request: Request,
    next: Next,
) -> Response {
    if (request.method() == Method::OPTIONS) {
        return next.run(request).await;
    }
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

async fn check_auth(auth: &HeaderValue, secrets: &Arc<RwLock<Secrets>>) -> bool {
    let auth_str = auth.to_str();
    if auth_str.is_err() {
        return false;
    }
    let auth = auth_str.unwrap();
    // Trim the leading "Bearer " from the auth string.
    let auth = auth.trim_start_matches("Bearer ");
    let secrets = secrets.read().await;
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
    match context.key_service.get_key(&key).await {
        Ok(value) => (StatusCode::OK, value.to_string()),
        Err(_) => (StatusCode::NOT_FOUND, "Not Found".to_owned()),
    }
}

async fn post_key(
    State(context): State<Arc<AppContext>>,
    Path(key): Path<String>,
    Json(value): Json<serde_json::Value>,
) -> impl IntoResponse {
    match context.key_service.post_key(&key, value.clone()).await {
        Ok(_) => (StatusCode::OK, value.to_string()),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_owned(),
        ),
    }
}

async fn put_key(
    State(context): State<Arc<AppContext>>,
    Path(key): Path<String>,
    Json(value): Json<serde_json::Value>,
) -> impl IntoResponse {
    match context.key_service.put_key(&key, value.clone()).await {
        Ok(_) => (StatusCode::OK, value.to_string()),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_owned(),
        ),
    }
}

async fn patch_key(
    State(context): State<Arc<AppContext>>,
    Path(key): Path<String>,
    Json(value): Json<serde_json::Value>,
) -> impl IntoResponse {
    match context.key_service.patch_key(&key, value.clone()).await {
        Ok(_) => (StatusCode::OK, value.to_string()),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_owned(),
        ),
    }
}

async fn list_keys(State(context): State<Arc<AppContext>>) -> impl IntoResponse {
    match context.key_service.list_keys().await {
        Ok(list) => (StatusCode::OK, serde_json::to_string(&list).unwrap()),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_owned(),
        ),
    }
}

async fn ws_key(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    State(context): State<Arc<AppContext>>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };

    println!("WS {key}: `{user_agent}` at connected.");
    ws.on_upgrade(move |socket| handle_websocket(socket, key,  context))
}
