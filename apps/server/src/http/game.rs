use crate::{
    http::{error::Error, extractor::AuthUser, Result},
    RoomState,
};
use aide::{
    axum::{
        routing::{get_with, post_with},
        ApiRouter,
    },
    transform::TransformOperation,
    NoApi,
};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    Json,
};
use futures::{stream::StreamExt, SinkExt};
use rand::{seq::SliceRandom, thread_rng};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::error::GenericError;

pub(crate) fn router() -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .api_route("/game/create", post_with(create_game, create_game_docs))
        .api_route("/game/:id", get_with(get_game, get_game_docs))
        .api_route("/game/:id", post_with(join_game, join_game_docs))
        .api_route("/game/ws", get_with(game_ws, game_ws_docs))
}

#[derive(Deserialize, JsonSchema)]
struct GameID {
    id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct GameBody<T> {
    game: T,
}

#[derive(Deserialize, JsonSchema)]
struct CreateGame {
    bet_value: i32,
    color_preference: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct Game {
    id: Uuid,
    white_player: Option<Uuid>,
    black_player: Option<Uuid>,
    bet_value: i32,
    moves: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct GamePlayer {
    id: Uuid,
    username: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct GameWithPlayers {
    id: Uuid,
    white_player: Option<GamePlayer>,
    black_player: Option<GamePlayer>,
    bet_value: i32,
    moves: Vec<String>,
}

async fn create_game(
    auth_user: AuthUser,
    state: State<crate::AppState>,
    payload: Json<GameBody<CreateGame>>,
) -> Result<Json<GameBody<Game>>> {
    let color_column = match payload.game.color_preference.as_deref() {
        Some("white") => "white_player",
        Some("black") => "black_player",
        _ => {
            let choices = ["white_player", "black_player"];
            *choices.choose(&mut thread_rng()).unwrap()
        }
    };

    let game_id: Uuid = sqlx::query(&format!(
        "INSERT INTO games ({color_column}, bet_value) VALUES ($1, $2) RETURNING id"
    ))
    .bind(auth_user.user_id)
    .bind(payload.game.bet_value)
    .fetch_one(&state.db)
    .await?
    .get("id");

    let mut white_player: Option<Uuid> = None;
    let mut black_player: Option<Uuid> = None;

    if color_column == "white_player" {
        white_player = Some(auth_user.user_id);
    } else {
        black_player = Some(auth_user.user_id);
    };

    let mut rooms = state.rooms.as_ref().lock().unwrap();
    let new_room = rooms
        .entry(game_id.to_string())
        .or_insert_with(RoomState::new);

    new_room
        .players
        .lock()
        .unwrap()
        .insert(auth_user.user_id.to_string());

    Ok(Json(GameBody {
        game: Game {
            id: game_id,
            white_player,
            black_player,
            bet_value: payload.game.bet_value,
            moves: vec![],
        },
    }))
}

fn create_game_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Create a game")
        .response::<200, Json<GameBody<Game>>>()
        .response::<400, Json<GenericError>>()
}

async fn join_game(
    auth_user: AuthUser,
    state: State<crate::AppState>,
    Path(GameID { id: game_id }): Path<GameID>,
) -> Result<Json<GameBody<GameWithPlayers>>> {
    let game_id = Uuid::parse_str(&game_id).map_err(|_| Error::BadRequest {
        message: "Invalid game id".to_string(),
    })?;

    let game = sqlx::query_as!(
        Game,
        r#"
            SELECT id, white_player, black_player, bet_value, moves
            FROM games
            WHERE id = $1
        "#,
        game_id,
    )
    .fetch_optional(&state.db)
    .await?;

    match game {
        Some(game) => {
            if game.black_player.is_some() && game.white_player.is_some() {
                return Err(Error::BadRequest {
                    message: "Game is full".to_string(),
                });
            }

            let color_column = if game.white_player.is_none() {
                "white_player"
            } else {
                "black_player"
            };

            let game_row = sqlx::query(&format!(
                r#"
                    UPDATE games
                    SET {color_column} = $1
                    WHERE id = $2
                    RETURNING id, white_player, black_player, bet_value, moves
                "#
            ))
            .bind(auth_user.user_id)
            .bind(game_id)
            .fetch_one(&state.db)
            .await?;

            let game = Game {
                id: game_row.get("id"),
                white_player: game_row.get("white_player"),
                black_player: game_row.get("black_player"),
                bet_value: game_row.get("bet_value"),
                moves: game_row.get("moves"),
            };

            let white_player = sqlx::query_as!(
                GamePlayer,
                r#"
                    SELECT id, username
                    FROM users
                    WHERE id = $1
                "#,
                game.white_player,
            )
            .fetch_optional(&state.db)
            .await?;

            let black_player = sqlx::query_as!(
                GamePlayer,
                r#"
                    SELECT id, username
                    FROM users
                    WHERE id = $1
                "#,
                game.black_player,
            )
            .fetch_optional(&state.db)
            .await?;

            let mut rooms = state.rooms.as_ref().lock().unwrap();

            if let Some(room) = rooms.get_mut(&game_id.to_string()) {
                let mut room_players = room.players.lock().unwrap();

                if room_players.len() > 2 {
                    return Err(Error::BadRequest {
                        message: "Game is full".to_string(),
                    });
                }

                room_players.insert(auth_user.user_id.to_string());
            }

            Ok(Json(GameBody {
                game: GameWithPlayers {
                    id: game.id,
                    white_player,
                    black_player,
                    bet_value: game.bet_value,
                    moves: game.moves,
                },
            }))
        }
        None => Err(Error::NotFound {
            message: "Game not found".to_string(),
        }),
    }
}

fn join_game_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Join a game")
        .response::<200, Json<GameBody<GameWithPlayers>>>()
        .response::<400, Json<GenericError>>()
        .response::<404, Json<GenericError>>()
}

async fn get_game(
    state: State<crate::AppState>,
    Path(GameID { id: game_id }): Path<GameID>,
) -> Result<Json<GameBody<GameWithPlayers>>> {
    let game_id = Uuid::parse_str(&game_id).map_err(|_| Error::BadRequest {
        message: "Invalid game ID".to_string(),
    })?;

    let game = sqlx::query_as!(
        Game,
        r#"
            SELECT id, white_player, black_player, bet_value, moves
            FROM games
            WHERE id = $1
        "#,
        game_id,
    )
    .fetch_optional(&state.db)
    .await?;

    match game {
        Some(game) => {
            let white_player = sqlx::query_as!(
                GamePlayer,
                r#"
                    SELECT id, username
                    FROM users
                    WHERE id = $1
                "#,
                game.white_player,
            )
            .fetch_optional(&state.db)
            .await?;

            let black_player = sqlx::query_as!(
                GamePlayer,
                r#"
                    SELECT id, username
                    FROM users
                    WHERE id = $1
                "#,
                game.black_player,
            )
            .fetch_optional(&state.db)
            .await?;

            Ok(Json(GameBody {
                game: GameWithPlayers {
                    id: game.id,
                    white_player,
                    black_player,
                    bet_value: game.bet_value,
                    moves: game.moves,
                },
            }))
        }
        None => Err(Error::NotFound {
            message: "Game not found".to_string(),
        }),
    }
}

fn get_game_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Get a game")
        .response::<200, Json<GameBody<GameWithPlayers>>>()
        .response::<400, Json<GenericError>>()
        .response::<404, Json<GenericError>>()
}

async fn game_ws(
    NoApi(ws): NoApi<WebSocketUpgrade>,
    State(state): State<crate::AppState>,
) -> NoApi<impl IntoResponse> {
    NoApi(ws.on_upgrade(|socket| game_handle_socket(socket, state)))
}

async fn game_handle_socket(socket: WebSocket, state: crate::AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut tx = None::<broadcast::Sender<String>>;

    while let Some(Ok(msg)) = receiver.next().await {
        let mut rooms = state.rooms.lock().unwrap();

        tx = match rooms.get_mut(msg.to_text().unwrap()) {
            Some(room) => Some(room.tx.clone()),
            None => None,
        };

        break;
    }

    let tx = tx.unwrap();
    let mut rx = tx.subscribe();

    let mut recv_messages = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            sender.send(Message::Text(msg)).await.unwrap();
        }
    });

    let mut send_messages = {
        let tx = tx.clone();
        tokio::spawn(async move {
            while let Some(Ok(Message::Text(text))) = receiver.next().await {
                dbg!(text.clone());

                let _ = tx.send(text);
            }
        })
    };

    tokio::select! {
        _ = (&mut send_messages) => recv_messages.abort(),
        _ = (&mut recv_messages) => send_messages.abort(),
    }
}

fn game_ws_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Websocket for game")
        .hidden(true)
}
