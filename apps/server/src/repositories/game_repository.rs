use crate::models::game::{Game, GameRecord, Player, PlayerColor};
use std::sync::Arc;

use crate::http::Result;
use crate::states::db;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

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
}

pub struct GameRepository {
    db: Arc<Pool<Postgres>>,
}

impl GameRepository {
    pub fn new() -> Self {
        Self { db: db::get() }
    }
}

impl GameRepositoryTrait for GameRepository {
    async fn get_player(&self, user_id: Uuid) -> sqlx::Result<Player> {
        sqlx::query_as!(
            Player,
            r#" SELECT id, username, email FROM users WHERE id = $1 "#,
            user_id,
        )
        .fetch_one(&*self.db)
        .await
    }

    async fn get_game(&self, game_id: Uuid) -> sqlx::Result<Game> {
        let game_record = sqlx::query_as!(
            GameRecord,
            r#" SELECT id, white_player, black_player, bet_value, moves FROM games WHERE id = $1 "#,
            game_id,
        )
        .fetch_one(&*self.db)
        .await?;

        let white_player = if let Some(player_id) = game_record.white_player {
            Some(self.get_player(player_id).await?)
        } else {
            None
        };

        let black_player = if let Some(player_id) = game_record.white_player {
            Some(self.get_player(player_id).await?)
        } else {
            None
        };

        let GameRecord {
            id,
            moves,
            bet_value,
            ..
        } = game_record;

        Ok(Game {
            id,
            white_player,
            black_player,
            bet_value,
            moves,
        })
    }

    async fn save_game(&self, game: GameRecord) -> Result<()> {
        sqlx::query!(
            r#" INSERT INTO games (id, white_player, black_player, bet_value, moves) VALUES ($1, $2, $3, $4, $5); "#,
            game.id,
            game.white_player,
            game.black_player,
            game.bet_value,
            &game.moves
        )
        .execute(&*self.db)
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
            "UPDATE games SET {} = $1 WHERE id = $2; ",
            player_color.to_string()
        ))
        .bind(player_id)
        .bind(game_id)
        .execute(&*self.db)
        .await?;

        Ok(())
    }
}
