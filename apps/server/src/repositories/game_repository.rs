use crate::http::Result;
use crate::models::{Game, GameState, Player};
use crate::states::db;
use mockall::automock;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(FromRow)]
struct GameRecord {
    id: Uuid,
    white_player: Uuid,
    black_player: Uuid,
    bet_value: i32,
    state: String,
    moves: Vec<String>,
}

impl GameRecord {
    fn to_game(self) -> Result<Game> {
        Ok(Game {
            id: self.id,
            white_player: self.white_player,
            black_player: self.black_player,
            bet_value: self.bet_value,
            state: GameState::from_str(&self.state)?,
            moves: self.moves,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Default)]
pub struct GameWithPlayers {
    pub id: Uuid,
    pub white_player: Player,
    pub black_player: Player,
    pub bet_value: i32,
    pub state: GameState,
    pub moves: Vec<String>,
}

#[automock]
pub trait GameRepositoryTrait {
    async fn get_player(&self, user_id: Uuid) -> Result<Player>;
    async fn get_game_with_players(&self, game_id: Uuid) -> Result<GameWithPlayers>;
    async fn get_game(&self, game_id: Uuid) -> Result<Game>;
    async fn save_game(&self, game: Game) -> Result<()>;
    async fn update_state(&self, game_id: Uuid, new_state: GameState) -> Result<()>;
    async fn record_move(&self, game_id: Uuid, move_played: String) -> Result<()>;
}

pub struct GameRepository {
    db: Pool<Postgres>,
}

impl GameRepository {
    pub fn new() -> Self {
        Self { db: db::get() }
    }
}

impl GameRepositoryTrait for GameRepository {
    async fn get_player(&self, user_id: Uuid) -> Result<Player> {
        Ok(
            sqlx::query_as::<_, Player>(r#" SELECT id, username, email FROM users WHERE id = $1 "#)
                .bind(user_id)
                .fetch_one(&self.db)
                .await?,
        )
    }

    async fn get_game_with_players(&self, game_id: Uuid) -> Result<GameWithPlayers> {
        let game = sqlx::query_as::<_, GameRecord>(
            r#" SELECT id, white_player, black_player, bet_value, moves, state FROM games WHERE id = $1 "#,
        )
        .bind(game_id)
        .fetch_one(&self.db)
        .await?.to_game()?;

        Ok(GameWithPlayers {
            id: game.id,
            white_player: self.get_player(game.white_player).await?,
            black_player: self.get_player(game.black_player).await?,
            state: game.state,
            bet_value: game.bet_value,
            moves: game.moves,
        })
    }

    async fn get_game(&self, game_id: Uuid) -> Result<Game> {
        let game = sqlx::query_as::<_, GameRecord>(
            r#" SELECT id, white_player, black_player, bet_value, moves, state FROM games WHERE id = $1 "#,
        )
        .bind(game_id)
        .fetch_one(&self.db)
        .await?.to_game()?;

        Ok(game)
    }

    async fn save_game(&self, game: Game) -> Result<()> {
        let result = sqlx::query(
            r#" INSERT INTO games (id, white_player, black_player, bet_value, moves) VALUES ($1, $2, $3, $4, $5); "#,
        )
        .bind(game.id)
        .bind(game.white_player)
        .bind(game.black_player)
        .bind(game.bet_value)
        .bind(&game.moves)
        .execute(&self.db)
        .await;

        result?;

        Ok(())
    }

    async fn update_state(&self, game_id: Uuid, new_state: GameState) -> Result<()> {
        sqlx::query(&format!("UPDATE games SET state = $1 WHERE id = $2;",))
            .bind(new_state.to_string())
            .bind(game_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    async fn record_move(&self, game_id: Uuid, move_played: String) -> Result<()> {
        sqlx::query(r#" UPDATE games SET moves = array_append(moves, $1) WHERE id = $2 "#)
            .bind(move_played)
            .bind(game_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}
