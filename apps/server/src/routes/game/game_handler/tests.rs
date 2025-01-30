use super::*;
use crate::models::game::{Game, GameState, Player};
use crate::repositories::game_repository::MockGameRepositoryTrait;
use play_move_service::MoveInfo;
use uuid::Uuid;

#[tokio::test]
async fn test_not_player_try_play_move() {
    let mut mock_game_repository = MockGameRepositoryTrait::new();

    mock_game_repository.expect_get_game().returning(|_| {
        Ok(Game {
            id: Uuid::new_v4(),
            white_player: Some(Player {
                id: Uuid::new_v4(),
                email: "".to_string(),
                username: "".to_string(),
            }),
            black_player: Some(Player {
                id: Uuid::new_v4(),
                email: "".to_string(),
                username: "".to_string(),
            }),
            state: GameState::Waiting,
            bet_value: 0,
            moves: vec![],
        })
    });
    let service = PlayMoveService::new(mock_game_repository);

    let input = MoveInfo {
        player_id: Uuid::new_v4(),
        game_id: Uuid::new_v4(),
        move_played: String::from("e4"),
    };

    let result = service.execute(input).await;

    assert!(result.is_err());
    assert_eq!(result, Err(String::from("You are not playing this game!")));
}

#[tokio::test]
async fn test_not_turned_player_try_play_move() {
    let mut mock_game_repository = MockGameRepositoryTrait::new();

    mock_game_repository.expect_get_game().returning(|_| {
        Ok(Game {
            id: Uuid::new_v4(),
            white_player: Some(Player {
                id: Uuid::new_v4(),
                email: "".to_string(),
                username: "".to_string(),
            }),
            black_player: Some(Player {
                id: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
                email: "".to_string(),
                username: "".to_string(),
            }),
            ..Default::default()
        })
    });

    let service = PlayMoveService::new(mock_game_repository);

    let input = MoveInfo {
        player_id: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
        game_id: Uuid::new_v4(),
        move_played: String::from("e4"),
    };

    let result = service.execute(input).await;

    assert!(result.is_err());
    assert_eq!(result, Err(String::from("It's not your turn!")));
}

#[tokio::test]
async fn test_right_player_play_move() {
    let mut mock_game_repository = MockGameRepositoryTrait::new();

    mock_game_repository.expect_get_game().returning(|_| {
        Ok(Game {
            id: Uuid::new_v4(),
            white_player: Some(Player {
                id: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
                email: "".to_string(),
                username: "".to_string(),
            }),
            black_player: Some(Player {
                id: Uuid::new_v4(),
                email: "".to_string(),
                username: "".to_string(),
            }),
            ..Default::default()
        })
    });

    mock_game_repository
        .expect_record_move()
        .once()
        .returning(|_, _| Ok(()));

    let service = PlayMoveService::new(mock_game_repository);

    let input = MoveInfo {
        player_id: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
        game_id: Uuid::new_v4(),
        move_played: String::from("e4"),
    };

    let result = service.execute(input).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_game_not_found() {
    let mut mock_game_repository = MockGameRepositoryTrait::new();

    mock_game_repository
        .expect_get_game()
        .once()
        .returning(|_| Err(sqlx::Error::RowNotFound));

    let input = MoveInfo {
        player_id: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
        game_id: Uuid::new_v4(),
        move_played: String::from("e4"),
    };

    let service = PlayMoveService::new(mock_game_repository);

    let result = service.execute(input).await;

    assert!(result.is_err());
    assert_eq!(result, Err(String::from("Game not found!")));
}
