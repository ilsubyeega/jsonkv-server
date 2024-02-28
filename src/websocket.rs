//
use std::{
    ops::ControlFlow,
    sync::{Arc, RwLock},
};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{context::AppContext, service::KeyServiceTrait};

pub struct ListenerContext {
    authorized: RwLock<bool>,
    listening: RwLock<Vec<String>>,
    sender: mpsc::Sender<ServerMessage>,
}
pub async fn handle_websocket(mut socket: WebSocket, context: Arc<AppContext>) {
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_err() {
        println!("Could not send ping!");
        return;
    }

    let (mut sender, mut receiver) = socket.split();

    // send the first message, auth.
    let serialized =
        serde_json::to_string(&ServerMessage::Auth("jsonkv-server".to_string())).unwrap();
    if sender.send(Message::Text(serialized)).await.is_err() {
        println!("client abruptly disconnected");
        return;
    }

    let (sender_channel_tx, mut sender_channel_rx) = mpsc::channel(512);
    let listener_context = Arc::new(ListenerContext {
        authorized: RwLock::new(false),
        listening: RwLock::new(Vec::new()),
        sender: sender_channel_tx,
    });

    // Receive task will receive messages from the websocket and process them
    let cloned = context.clone();
    let listener_cloned = listener_context.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if process_message(&listener_cloned, &cloned, msg)
                .await
                .is_break()
            {
                break;
            }
        }
    });

    // Send task will send messages to the websocket
    let mut send_task = tokio::spawn(async move {
        // listen
        while let Some(i) = sender_channel_rx.recv().await {
            let serialized = serde_json::to_string(&i).unwrap();
            if sender.send(Message::Text(serialized)).await.is_err() {
                println!("client abruptly disconnected");
                break;
            }
        }
    });

    // Listen for key changes, and then send them to the client via send_task.
    let mut listen_key_task = tokio::spawn(async move {
        let sender = listener_context.sender.clone();
        let mut receiver = context.broadcast.subscribe();
        while let Ok(i) = receiver.recv().await {
            let (key, value) = i;
            if listener_context.listening.read().unwrap().contains(&key) {
                sender
                    .send(ServerMessage::Data { key, value })
                    .await
                    .unwrap();
            }
        }
    });

    tokio::select! {
        _ = &mut recv_task => {
            send_task.abort();
            listen_key_task.abort();
        },
        _ = &mut send_task => {
            recv_task.abort();
            listen_key_task.abort();
        },
        _ = &mut listen_key_task => {
            recv_task.abort();
            send_task.abort();
        }
    }

    println!("client disconnected");
}

async fn process_message(
    context: &Arc<ListenerContext>,
    app_context: &Arc<AppContext>,
    msg: Message,
) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            // parse message and if message is not valid, continue
            let msg: Result<ClientMessage, serde_json::Error> = serde_json::from_str(&t);
            if let Err(err) = msg {
                // unable to parse message
                context.sender.send(ServerMessage::Error {
                    message: err.to_string(),
                }).await.unwrap();
                return ControlFlow::Continue(());
            }

            // check if the client is authorized
            let is_authorized = *context.authorized.read().unwrap();
            if !is_authorized {
                let msg = msg.unwrap();
                match msg {
                    ClientMessage::Authenticate(secret) => {
                        if app_context.secrets.read().await.contains_key(&secret) {
                            *context.authorized.write().unwrap() = true;
                            context.sender.send(ServerMessage::Authenticated).await.unwrap();
                            println!("client authorized");
                        } else {
                            println!("client unauthorized");
                        }
                    }
                    _ => {
                        println!("client unauthorized");
                    }
                }
                return ControlFlow::Continue(());
            } else {
                match msg.unwrap() {
                    ClientMessage::Subscribe(keys) => {
                        // push keys to listening
                        {
                            let mut listening = context.listening.write().unwrap();
                            for key in &keys {
                                if !listening.contains(key) {
                                    listening.push(key.clone());
                                }
                            }
                        }
                        // return the keys and their values
                        for key in keys {
                            let value = app_context.key_service.get_key(&key).await;
                            if let Err(err) = value {
                                context.sender.send(ServerMessage::Error {
                                    message: err.to_string(),
                                }).await.unwrap();
                            } else {
                                context.sender.send(ServerMessage::Subscribed {
                                    key: key.clone(),
                                    value: value.unwrap(),
                                }).await.unwrap();
                            }
                        }
                    }
                    ClientMessage::Data { key, value } => {
                        if context.listening.read().unwrap().contains(&key) {
                            let req = app_context.key_service.put_key(&key, value).await;
                            if let Err(err) = req {
                                context.sender.send(ServerMessage::Error {
                                    message: err.to_string(),
                                }).await.unwrap();
                            }
                        }
                    }
                    ClientMessage::Patch { key, value } => {
                        if context.listening.read().unwrap().contains(&key) {
                            let req = app_context.key_service.patch_key(&key, value).await;
                            if let Err(err) = req {
                                context.sender.send(ServerMessage::Error {
                                    message: err.to_string(),
                                }).await.unwrap();
                            }
                        }
                    }
                    _ => {}
                }
            }
            println!("client sent: {}", t);
        }
        Message::Close(_) => {
            return ControlFlow::Break(());
        }
        _ => {}
    }
    ControlFlow::Continue(())
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
enum ServerMessage {
    Auth(String),
    Authenticated,
    Subscribed {
        key: String,
        value: serde_json::Value,
    },
    Data {
        key: String,
        value: serde_json::Value,
    },
    Error {
        message: String, // invalid-message or so...
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum ClientMessage {
    Authenticate(String),
    Subscribe(Vec<String>),
    Data {
        key: String,
        value: serde_json::Value,
    },
    Patch {
        key: String,
        value: serde_json::Value,
    },
}
