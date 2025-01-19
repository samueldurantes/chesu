use crate::http::{extractor::AuthUser, Result};
use crate::{http::error::Error, RoomState};
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

const DEFAULT_BET_VALUE: i32 = 10;

pub(crate) fn router() -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .api_route(
            "/game/pairing",
            post_with(quick_pairing_game, quick_pairing_game_docs),
        )
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

#[derive(Default, Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Game {
    pub id: Uuid,
    pub white_player: Option<Uuid>,
    pub black_player: Option<Uuid>,
    pub last_move_player: Option<Uuid>,
    pub bet_value: i32,
    pub moves: Vec<String>,
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

async fn quick_pairing_game(
    auth_user: AuthUser,
    state: State<crate::AppState>,
) -> Result<Json<GameBody<Uuid>>> {
    let current_game = {
        let mut available_game = state.available_game.lock().unwrap();
        let current_game = available_game.clone().unwrap_or(Uuid::new_v4());

        *available_game = if (*available_game).is_some() {
            None
        } else {
            Some(current_game)
        };

        current_game
    };

    let username: String = sqlx::query(
        "WITH inserted_game AS (
            INSERT INTO games (id, white_player, bet_value) VALUES ($1, $2, $3) ON CONFLICT (id) DO UPDATE SET black_player = $2
        )
        SELECT (username) FROM users WHERE id = $2",
    )
    .bind(current_game)
    .bind(auth_user.user_id)
    .bind(DEFAULT_BET_VALUE)
    .fetch_one(&state.db)
    .await?.get(0);

    let mut rooms = state.rooms.as_ref().lock().unwrap();
    let room = rooms
        .entry(current_game.to_string())
        .or_insert_with(RoomState::new);
    let mut room_players = room.players.lock().unwrap();

    room_players.insert(auth_user.user_id.to_string());

    if room_players.len() == 2 {
        notify_other_player(&room, username);
    }

    Ok(Json(GameBody { game: current_game }))
}

fn quick_pairing_game_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Quick Pairing")
        .description("Quick Pair players to play")
        .response::<200, Json<GameBody<Uuid>>>()
        .response::<400, Json<GenericError>>()
}

fn pick_color(color_preference: Option<&str>) -> String {
    match color_preference {
        Some("white") => "white_player",
        Some("black") => "black_player",
        _ => {
            let choices = ["white_player", "black_player"];
            *choices.choose(&mut thread_rng()).unwrap()
        }
    }
    .to_string()
}

async fn create_game(
    auth_user: AuthUser,
    state: State<crate::AppState>,
    payload: Json<GameBody<CreateGame>>,
) -> Result<Json<GameBody<Uuid>>> {
    let game_id: Uuid = sqlx::query(&format!(
        "INSERT INTO games ({}, bet_value) VALUES ($1, $2) RETURNING id",
        pick_color(payload.game.color_preference.as_deref())
    ))
    .bind(auth_user.user_id)
    .bind(payload.game.bet_value)
    .fetch_one(&state.db)
    .await?
    .get("id");

    let mut rooms = state.rooms.as_ref().lock().unwrap();
    let new_room = rooms
        .entry(game_id.to_string())
        .or_insert_with(RoomState::new);

    new_room
        .players
        .lock()
        .unwrap()
        .insert(auth_user.user_id.to_string());

    Ok(Json(GameBody { game: game_id }))
}

fn create_game_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Create a game")
        .response::<200, Json<GameBody<Uuid>>>()
        .response::<400, Json<GenericError>>()
}

fn notify_other_player(room: &RoomState, username: String) {
    room.tx
        .send(
            serde_json::json!({
               "event": "join",
               "data": {
                 "username": username
                }
            })
            .to_string(),
        )
        .expect("Error on notify other player!");
}

async fn join_game(
    auth_user: AuthUser,
    state: State<crate::AppState>,
    Path(GameID { id: game_id }): Path<GameID>,
) -> Result<Json<GameBody<Uuid>>> {
    let game_id = Uuid::parse_str(&game_id).map_err(|_| Error::BadRequest {
        message: "Invalid game id".to_string(),
    })?;

    let username = sqlx::query!(
        r#"
        WITH updated_game AS (
            UPDATE games
            SET 
                white_player = COALESCE(white_player, $2),
                black_player = COALESCE(black_player, $2)
            WHERE id = $1
            RETURNING id
        )
        SELECT u.username
        FROM updated_game g
        JOIN users u ON u.id = $2
    "#,
        game_id,
        auth_user.user_id
    )
    .fetch_one(&state.db)
    .await?
    .username;

    let mut rooms = state.rooms.as_ref().lock().unwrap();

    if let Some(room) = rooms.get_mut(&game_id.to_string()) {
        let mut room_players = room.players.lock().unwrap();

        if room_players.len() > 2 {
            return Err(Error::BadRequest {
                message: "Game is full".to_string(),
            });
        }

        room_players.insert(auth_user.user_id.to_string());
        notify_other_player(&room, username);
    }

    Ok(Json(GameBody { game: game_id }))
}

fn join_game_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Join a game")
        .response::<200, Json<GameBody<Uuid>>>()
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
            SELECT id, white_player, black_player, bet_value, last_move_player, moves
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

// TODO: Think in a better name for this struct
#[derive(Clone, Serialize, Deserialize)]
struct Move {
    game_id: Uuid,
    player_id: Uuid,
    play_move: String,
}

async fn update_moves(game: Game, move_: Move, state: &crate::AppState) -> Result<Game, String> {
    if game
        .last_move_player
        .filter(|&id| id == move_.player_id)
        .is_some()
    {
        return Err("Not your turn".to_string());
    }

    // TODO: Use Redis to save the moves rather than call the DB every move
    sqlx::query_as!(
        Game,
        r#"
            UPDATE games
            SET 
                last_move_player = $1,
                moves = array_append(moves, $3)
            WHERE id = $2
            RETURNING id, white_player, black_player, bet_value, last_move_player, moves
        "#,
        move_.player_id,
        game.id,
        move_.play_move,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| "Failed to update moves".to_string())
}

async fn handle_board_event(message: String, state: &crate::AppState) -> Result<Move, String> {
    let move_: Move =
        serde_json::from_str(&message).map_err(|_| "Failed to build the move".to_string())?;

    let game = sqlx::query_as!(
        Game,
        r#"
            SELECT id, white_player, black_player, bet_value, last_move_player, moves
            FROM games
            WHERE id = $1
        "#,
        move_.game_id,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| "Failed to get the game".to_string())?;

    update_moves(game, move_.clone(), state).await?;

    Ok(move_)
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
            match sender.send(Message::Text(msg)).await {
                _ => (),
            }
        }
    });

    let mut send_messages = {
        let tx = tx.clone();
        tokio::spawn(async move {
            while let Some(Ok(Message::Text(message))) = receiver.next().await {
                match handle_board_event(message, &state).await {
                    Ok(move_) => {
                        let Ok(move_json) = serde_json::to_string(&move_) else {
                            let _ = tx.send("Failed to build the move".to_string());
                            return ();
                        };

                        let _ = tx.send(move_json);
                    }
                    // TODO: It should be return to client
                    Err(_) => (),
                }
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
