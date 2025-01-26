use crate::{
    models::rooms_manager::RoomsManagerTrait,
    repositories::game_repository::GameRepository,
    services::game::play_move_service::{MoveInfo, PlayMoveService},
};
use aide::{transform::TransformOperation, NoApi};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::SinkExt;
use futures::StreamExt;
use tokio::sync::broadcast;
use tracing::info;

pub fn resource() -> PlayMoveService<GameRepository> {
    PlayMoveService::new(GameRepository::new())
}

pub async fn route(
    play_move_service: PlayMoveService<GameRepository>,
    NoApi(ws): NoApi<WebSocketUpgrade>,
) -> NoApi<impl IntoResponse> {
    NoApi(ws.on_upgrade(|socket| game_handler(socket, play_move_service)))
}

fn connect_channel(
    room_id: String,
) -> Option<(broadcast::Sender<String>, broadcast::Receiver<String>)> {
    let rooms_manager = crate::models::rooms_manager::RoomsManager::new();
    let tx = rooms_manager.get_room_tx(uuid::Uuid::parse_str(&room_id).unwrap());

    tx.map(|tx| (tx.clone(), tx.subscribe()))
}

async fn game_handler(socket: WebSocket, play_move_service: PlayMoveService<GameRepository>) {
    let (mut sender, mut receiver) = socket.split();
    let mut channel = None::<(broadcast::Sender<String>, broadcast::Receiver<String>)>;

    while let Some(Ok(Message::Text(room_id))) = receiver.next().await {
        channel = connect_channel(room_id);
        break;
    }

    let (tx, mut rx) = channel.unwrap();

    let mut relay_messages = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            sender.send(Message::Text(msg)).await.unwrap();
        }
    });

    let mut process_received_messages = {
        tokio::spawn(async move {
            while let Some(Ok(Message::Text(move_info))) = receiver.next().await {
                let play_result = play_move_service
                    .execute(MoveInfo::from_str(&move_info).unwrap())
                    .await;

                match play_result {
                    Ok(()) => tx.send(move_info),
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
