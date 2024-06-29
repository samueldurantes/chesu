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
};
use axum::{
    extract::{Path, State},
    Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::GenericError;

pub(crate) fn router() -> ApiRouter<crate::AppState> {
    ApiRouter::new()
        .api_route("/game/create", post_with(create_game, create_game_docs))
        .api_route("/game/:id", get_with(get_game, get_game_docs))
        .api_route("/game/:id", post_with(join_game, join_game_docs))
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
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct Game {
    id: Uuid,
    white_player: Uuid,
    black_player: Option<Uuid>,
    bet_value: i32,
    moves: Vec<String>,
}

async fn create_game(
    auth_user: AuthUser,
    state: State<crate::AppState>,
    payload: Json<GameBody<CreateGame>>,
) -> Result<Json<GameBody<Game>>> {
    let game = sqlx::query_scalar!(
        r#"
            INSERT INTO games (white_player, bet_value) VALUES ($1, $2) RETURNING id
        "#,
        auth_user.user_id,
        payload.game.bet_value,
    )
    .fetch_optional(&state.db)
    .await?;

    match game {
        Some(game_id) => {
            let mut rooms = state.rooms.as_ref().lock().unwrap();
            let new_room = rooms.insert(game_id.to_string(), RoomState::new());

            if let Some(room) = new_room {
                room.players
                    .lock()
                    .unwrap()
                    .insert(auth_user.user_id.to_string());
            }

            Ok(Json(GameBody {
                game: Game {
                    id: game_id,
                    white_player: auth_user.user_id,
                    black_player: None,
                    bet_value: payload.game.bet_value,
                    moves: vec![],
                },
            }))
        }
        None => Err(Error::BadRequest {
            message: "Error when creating game".to_string(),
        }),
    }
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
) -> Result<Json<GameBody<Game>>> {
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
            if game.black_player.is_some() {
                return Err(Error::BadRequest {
                    message: "Game is full".to_string(),
                });
            }

            let game = sqlx::query_as!(
                Game,
                r#"
                    UPDATE games
                    SET black_player = $1
                    WHERE id = $2
                    RETURNING id, white_player, black_player, bet_value, moves
                "#,
                auth_user.user_id,
                game_id,
            )
            .fetch_one(&state.db)
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

            Ok(Json(GameBody { game }))
        }
        None => Err(Error::NotFound {
            message: "Game not found".to_string(),
        }),
    }
}

fn join_game_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Join a game")
        .response::<200, Json<GameBody<Game>>>()
        .response::<400, Json<GenericError>>()
        .response::<404, Json<GenericError>>()
}

async fn get_game(
    state: State<crate::AppState>,
    Path(GameID { id: game_id }): Path<GameID>,
) -> Result<Json<GameBody<Game>>> {
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
        Some(game) => Ok(Json(GameBody { game })),
        None => Err(Error::NotFound {
            message: "Game not found".to_string(),
        }),
    }
}

fn get_game_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Get a game")
        .response::<200, Json<GameBody<Game>>>()
        .response::<400, Json<GenericError>>()
        .response::<404, Json<GenericError>>()
}
