use crate::http::{error::Error, extractor::AuthUser, Result};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct GameBody<T> {
    pub game: T,
}

#[derive(Deserialize)]
pub struct CreateGame {
    pub bet_value: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub id: Uuid,
    pub white_player: String,
    pub black_player: Option<Uuid>,
    pub bet_value: i32,
    pub moves: Vec<String>,
}

// Make a cURL for this route
// curl -X POST http://localhost:3000/game/create -H "Content-Type: application/json" -d '{"game": {"bet_value": 100}}'
pub async fn create_game(
    auth_user: AuthUser,
    context: State<crate::Context>,
    payload: Json<GameBody<CreateGame>>,
) -> Result<Json<GameBody<Game>>> {
    let game = sqlx::query_scalar!(
        r#"
            INSERT INTO games (white_player, bet_value) VALUES ($1, $2) RETURNING id
        "#,
        auth_user.user_id,
        payload.game.bet_value,
    )
    .fetch_optional(&context.db)
    .await?;

    match game {
        Some(game_id) => {
            Ok(Json(GameBody {
                game: Game {
                    id: game_id,
                    white_player: auth_user.user_id.to_string(),
                    black_player: None,
                    bet_value: payload.game.bet_value,
                    moves: vec![],
                }
            }))
        },
        None => Err(Error::BadRequest {
            error: "Error when creating game".to_string(),
        }),
    }
}

// JWT -> eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3MTY4NjI5MzAsImV4cCI6MTcxOTQ1NDkzMCwibmJmIjoxNzE2ODYyOTMwLCJ1c2VyX2lkIjoiMTY2ZDY0YjItMWM5OS0xMWVmLTg1ODMtYjdhM2UxYjVjYjkyIn0.pFaL-73MVUlg-88tts7FMfutZWYS4NE7MO6C4NKnNDU"
// Make a cURL for this route
// curl -X GET http://localhost:3000/game/166d64b2-1c99-11ef-8583-b7a3e1b5cb
pub async fn get_game(
    context: State<crate::Context>,
    game_id: String,
) -> Result<Json<GameBody<Game>>> {
    let game_id = Uuid::parse_str(&game_id).map_err(|_| Error::BadRequest {
        error: "Invalid game id".to_string(),
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
    .fetch_optional(&context.db)
    .await?;

    match game {
        Some(game) => {
            Ok(Json(GameBody {
                game
            }))
        },
        None => Err(Error::NotFound {
            error: "Game not found".to_string(),
        }),
    }
}
