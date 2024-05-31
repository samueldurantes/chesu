use crate::http::{error::Error, extractor::AuthUser, Result};
use aide::{
    axum::{
        routing::{get_with, post_with},
        ApiRouter,
    },
    transform::TransformOperation,
};
use axum::{extract::State, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(crate) fn router() -> ApiRouter<crate::State> {
    ApiRouter::new()
        .api_route("/game/create", post_with(create_game, create_game_docs))
        .api_route("/game/:id", get_with(get_game, get_game_docs))
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
    state: State<crate::State>,
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
        Some(game_id) => Ok(Json(GameBody {
            game: Game {
                id: game_id,
                white_player: auth_user.user_id,
                black_player: None,
                bet_value: payload.game.bet_value,
                moves: vec![],
            },
        })),
        None => Err(Error::BadRequest {
            error: "Error when creating game".to_string(),
        }),
    }
}

fn create_game_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Create a game")
        .response::<200, Json<GameBody<Game>>>()
}

async fn get_game(state: State<crate::State>, game_id: String) -> Result<Json<GameBody<Game>>> {
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
    .fetch_optional(&state.db)
    .await?;

    match game {
        Some(game) => Ok(Json(GameBody { game })),
        None => Err(Error::NotFound {
            error: "Game not found".to_string(),
        }),
    }
}

fn get_game_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Game")
        .description("Get a game")
        .response::<200, Json<GameBody<Game>>>()
}
