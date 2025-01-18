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

impl Game {
    fn new(bet_value: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            bet_value,
            ..Default::default()
        }
    }

    fn add_player(mut self, player: Uuid) -> Self {
        match (self.white_player, self.black_player) {
            (None, Some(_)) => {
                self.white_player = Some(player);
            }
            (Some(_), None) => {
                self.black_player = Some(player);
            }
            _ => (),
        }

        self
    }

    fn has_two_players(&self) -> bool {
        self.white_player.is_some() && self.black_player.is_some()
    }
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
    let game = {
        let mut room = state.pairing_room.game.lock().unwrap();
        if room.is_some() {
            let game = <std::option::Option<Game> as Clone>::clone(&room)
                .unwrap()
                .add_player(auth_user.user_id);
            state.pairing_room.notifier.notify_waiters();

            *room = None;

            game
        } else {
            let new_game = Game::new(DEFAULT_BET_VALUE).add_player(auth_user.user_id);
            *room = Some(new_game.clone());
            new_game
        }
    };

    let wait_future = async {
        if game.has_two_players() {
            sqlx::query(&format!( "INSERT INTO games (id, white_player, black_player, bet_value) VALUES ($1, $2, $3, $4) "))
            .bind(game.id)
            .bind(game.white_player.unwrap())
            .bind(game.black_player.unwrap())
            .bind(game.bet_value)
            .fetch_optional(&state.db)
            .await
            .unwrap();
        } else {
            state.pairing_room.notifier.notified().await;
        }

        let mut rooms = state.rooms.as_ref().lock().unwrap();
        let new_room = rooms
            .entry(game.id.to_string())
            .or_insert_with(RoomState::new);

        new_room
            .players
            .lock()
            .unwrap()
            .insert(auth_user.user_id.to_string());
    };

    wait_future.await;

    Ok(Json(GameBody { game: game.id }))
}

fn quick_pairing_game_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Quick Pairing")
        .description("Quick Pair players to play")
        .response::<200, Json<GameBody<Uuid>>>()
        .response::<400, Json<GenericError>>()
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
            last_move_player: None,
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
                last_move_player: None,
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
