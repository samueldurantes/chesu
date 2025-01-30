use crate::models::game::{Game, GameRecord, GameState, Player, PlayerColor};

use crate::http::Result;
use crate::states::db;
use mockall::automock;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[automock]
pub trait GameRepositoryTrait {
    async fn get_player(&self, user_id: Uuid) -> sqlx::Result<Player>;
    async fn get_game(&self, game_id: Uuid) -> sqlx::Result<Game>;
    async fn save_game(&self, game: GameRecord) -> Result<()>;
    async fn add_player(
        &self,
        game_id: Uuid,
        player_id: Uuid,
        player_color: PlayerColor,
    ) -> Result<()>;
    async fn record_move(&self, game_id: Uuid, move_played: String) -> sqlx::Result<()>;
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
    async fn get_player(&self, user_id: Uuid) -> sqlx::Result<Player> {
        sqlx::query_as::<_, Player>(r#" SELECT id, username, email FROM users WHERE id = $1 "#)
            .bind(user_id)
            .fetch_one(&self.db)
            .await
    }

    async fn get_game(&self, game_id: Uuid) -> sqlx::Result<Game> {
        let game_record = sqlx::query_as::<_, GameRecord>(
            r#" SELECT id, white_player, black_player, bet_value, moves, state FROM games WHERE id = $1 "#,
        )
        .bind(game_id)
        .fetch_one(&self.db)
        .await?;

        let white_player = if let Some(player_id) = game_record.white_player {
            Some(self.get_player(player_id).await?)
        } else {
            None
        };

        let black_player = if let Some(player_id) = game_record.black_player {
            Some(self.get_player(player_id).await?)
        } else {
            None
        };

        let GameRecord {
            id,
            moves,
            state,
            bet_value,
            ..
        } = game_record;

        Ok(Game {
            id,
            white_player,
            black_player,
            state: GameState::from_str(&state),
            bet_value,
            moves,
        })
    }

    async fn save_game(&self, game: GameRecord) -> Result<()> {
        sqlx::query(
            r#" INSERT INTO games (id, white_player, black_player, bet_value, moves, state) VALUES ($1, $2, $3, $4, $5); "#,
        )
        .bind(game.id)
        .bind(game.white_player)
        .bind(game.black_player)
        .bind(game.bet_value)
        .bind(&game.moves)
        .bind(game.state)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn add_player(
        &self,
        game_id: Uuid,
        player_id: Uuid,
        player_color: PlayerColor,
    ) -> Result<()> {
        sqlx::query(&format!(
            "UPDATE games SET {} = $1 WHERE id = $2;",
            player_color.to_string()
        ))
        .bind(player_id)
        .bind(game_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn record_move(&self, game_id: Uuid, move_played: String) -> sqlx::Result<()> {
        sqlx::query(r#" UPDATE games SET moves = array_append(moves, $1) WHERE id = $2 "#)
            .bind(move_played)
            .bind(game_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}
