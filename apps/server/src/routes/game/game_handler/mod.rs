use crate::{
    models::{Event, RoomsManager, RoomsManagerTrait},
    repositories::GameRepository,
};
use aide::{transform::TransformOperation, NoApi};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
};
use disconnect_service::DisconnectService;
use futures::SinkExt;
use futures::StreamExt;
use play_move_service::PlayMoveService;
use tokio::sync::broadcast;
use tracing;

mod disconnect_service;
mod play_move_service;

fn resource() -> (
    PlayMoveService<GameRepository, RoomsManager>,
    DisconnectService<GameRepository, RoomsManager>,
) {
    (
        PlayMoveService::new(GameRepository::new(), RoomsManager::new()),
        DisconnectService::new(GameRepository::new(), RoomsManager::new()),
    )
}

pub async fn route(NoApi(ws): NoApi<WebSocketUpgrade>) -> NoApi<impl IntoResponse> {
    NoApi(ws.on_upgrade(game_handler))
}

fn connect_channel(
    room_id: String,
) -> Option<(broadcast::Sender<String>, broadcast::Receiver<String>)> {
    let rooms_manager = crate::models::RoomsManager::new();
    let tx = rooms_manager
        .get_room_tx(uuid::Uuid::parse_str(&room_id).unwrap())
        .ok();

    tx.map(|tx| (tx.clone(), tx.subscribe()))
}

async fn game_handler(socket: WebSocket) {
    let (play_move, disconnect) = resource();

    let (mut sender, mut receiver) = socket.split();
    let mut channel = None::<(broadcast::Sender<String>, broadcast::Receiver<String>)>;

    while let Some(Ok(Message::Text(room_id))) = receiver.next().await {
        channel = connect_channel(room_id);
        break;
    }

    let (tx, mut rx) = channel.unwrap();

    let mut relay_messages = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            sender.send(Message::Text(msg)).await.unwrap_or(());
        }
    });

    let mut process_received_messages = {
        tokio::spawn(async move {
            while let Some(Ok(Message::Text(json_event))) = receiver.next().await {
                tracing::info!("{json_event}");

                let result = match Event::from_json(&json_event) {
                    Ok(Event::PlayMove(data)) => play_move.execute(data).await,
                    Ok(Event::Disconnect(data)) => disconnect.execute(data).await,
                    _ => Err(String::from("Could not build event!")),
                };

                match result {
                    Ok(()) => tx.send(json_event),
                    Err(err_msg) => tx.send(err_msg),
                }
                .unwrap();
            }
        })
    };

    tokio::select! {
        _ = (&mut process_received_messages) => relay_messages.abort(),
        _ = (&mut relay_messages) => process_received_messages.abort(),
    }
}

pub fn docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Websocket for game")
        .hidden(true)
}
