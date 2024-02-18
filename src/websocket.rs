//
use std::{net::SocketAddr, ops::ControlFlow, sync::Arc};

use axum::{
    extract::ws::{Message, WebSocket},
    serve::Serve,
};
use futures::{stream::StreamExt, SinkExt};
use serde::{Deserialize, Serialize};

use crate::{context::AppContext, service::KeyServiceTrait};

pub async fn handle_websocket(mut socket: WebSocket, key: String, context: Arc<AppContext>) {
    if !socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        println!("Could not send ping!");
        return;
    }

    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if process_message(msg).is_break() {
                return;
            }
        } else {
            println!("client  abruptly disconnected");
            return;
        }
    }

    let (mut sender, mut receiver) = socket.split();

    // send subscribed message
    let value = context
        .key_service
        .get_key(&key)
        .await
        .unwrap_or(serde_json::Value::Null);
    let serialized = serde_json::to_string(&ServerMessage::Subscribed {
        key: key.clone(),
        value: value.clone(),
    })
    .unwrap();
    if sender.send(Message::Text(serialized)).await.is_err() {
        println!("client abruptly disconnected");
        return;
    }

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if process_message(msg).is_break() {
                break;
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        let mut receiver = context.broadcast.subscribe();
        while let Ok(msg) = receiver.recv().await {
            if key != msg.0 {
                continue;
            }
            let serialized = serde_json::to_string(&ServerMessage::Data {
                key: msg.0,
                value: msg.1,
            })
            .unwrap();
            if sender.send(Message::Text(serialized)).await.is_err() {
                println!("client abruptly disconnected");
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut recv_task => {
            send_task.abort();
        },
        _ = &mut send_task => {
            recv_task.abort();
        }
    }

    println!("client disconnected");
}

fn process_message(msg: Message) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!("client sent: {t}");
        }
        Message::Close(_) => {
            return ControlFlow::Break(());
        }
        _ => {}
    }
    ControlFlow::Continue(())
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum ServerMessage {
    Subscribed {
        key: String,
        value: serde_json::Value,
    },
    Data {
        key: String,
        value: serde_json::Value,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ClientMessage {
    Data {
        key: String,
        value: serde_json::Value,
    },
}
