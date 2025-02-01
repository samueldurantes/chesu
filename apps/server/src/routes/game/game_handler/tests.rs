use super::*;
use crate::models::{
    event::{DisconnectInfo, MoveInfo},
    game::{Game, GameState, Player},
    rooms_manager::{MockRoomsManagerTrait, Room},
};
use crate::repositories::game_repository::MockGameRepositoryTrait;
use uuid::Uuid;

#[tokio::test]
async fn test_not_player_try_play_move() {
    let mut mock_game_repository = MockGameRepositoryTrait::new();
    let mock_rooms_manager = MockRoomsManagerTrait::new();

    mock_game_repository.expect_get_game().returning(|_| {
        Ok(Game {
            id: Uuid::new_v4(),
            white_player: Uuid::new_v4(),
            black_player: Uuid::new_v4(),
            state: GameState::Waiting,
            bet_value: 0,
            moves: vec![],
        })
    });
    let service = PlayMoveService::new(mock_game_repository, mock_rooms_manager);

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
    let mock_rooms_manager = MockRoomsManagerTrait::new();

    mock_game_repository.expect_get_game().returning(|_| {
        Ok(Game {
            id: Uuid::new_v4(),
            white_player: Uuid::new_v4(),
            black_player: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
            ..Default::default()
        })
    });

    let service = PlayMoveService::new(mock_game_repository, mock_rooms_manager);

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
    let mock_rooms_manager = MockRoomsManagerTrait::new();

    mock_game_repository.expect_get_game().returning(|_| {
        Ok(Game {
            id: Uuid::new_v4(),
            white_player: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
            black_player: Uuid::new_v4(),
            ..Default::default()
        })
    });

    mock_game_repository
        .expect_record_move()
        .once()
        .returning(|_, _| Ok(()));

    let service = PlayMoveService::new(mock_game_repository, mock_rooms_manager);

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
    let mock_rooms_manager = MockRoomsManagerTrait::new();

    mock_game_repository
        .expect_get_game()
        .once()
        .returning(|_| Err(sqlx::Error::RowNotFound));

    let input = MoveInfo {
        player_id: uuid::uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a"),
        game_id: Uuid::new_v4(),
        move_played: String::from("e4"),
    };

    let service = PlayMoveService::new(mock_game_repository, mock_rooms_manager);

    let result = service.execute(input).await;

    assert!(result.is_err());
    assert_eq!(result, Err(String::from("Game not found!")));
}

#[tokio::test]
async fn test_player_disconnection_from_request() {
    let mock_game_repository = MockGameRepositoryTrait::new();
    let mut mock_rooms_manager = MockRoomsManagerTrait::new();

    mock_rooms_manager.expect_get_room().once().returning(|_| {
        Some(Room {
            request_key: String::from("w-10-0-0"),
            white_player: Some(Player {
                id: uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"),
                ..Default::default()
            }),
            black_player: None,
            tx: broadcast::channel(100).0,
        })
    });

    mock_rooms_manager
        .expect_remove_request()
        .once()
        .withf(|request_key| request_key == "w-10-0-0")
        .returning(|_| ());

    mock_rooms_manager
        .expect_remove_room()
        .once()
        .withf(|id| id == &uuid::uuid!("6a2b4680-e96d-4e33-923f-3979d09d8ade"))
        .returning(|_| ());

    let input = DisconnectInfo {
        game_id: uuid::uuid!("6a2b4680-e96d-4e33-923f-3979d09d8ade"),
        player_id: uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"),
    };

    let service = DisconnectService::new(mock_game_repository, mock_rooms_manager);

    let result = service.execute(input).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_player_disconnection_from_game() {
    let mut mock_game_repository = MockGameRepositoryTrait::new();
    let mut mock_rooms_manager = MockRoomsManagerTrait::new();

    let input = DisconnectInfo {
        game_id: uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"),
        player_id: uuid::uuid!("6a2b4680-e96d-4e33-923f-3979d09d8ade"),
    };

    mock_rooms_manager
        .expect_get_room()
        .once()
        .withf(|id| id == &uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"))
        .returning(|_| {
            Some(Room {
                request_key: String::from("w-10-0-0"),
                white_player: Some(Player {
                    id: uuid::uuid!("6a2b4680-e96d-4e33-923f-3979d09d8ade"),
                    ..Default::default()
                }),
                black_player: Some(Player {
                    id: Uuid::new_v4(),
                    ..Default::default()
                }),
                tx: broadcast::channel(100).0,
            })
        });

    mock_game_repository
        .expect_get_game()
        .once()
        .withf(|id| id == &uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"))
        .returning(|id| {
            Ok(Game {
                id,
                white_player: uuid::uuid!("6a2b4680-e96d-4e33-923f-3979d09d8ade"),
                black_player: Uuid::new_v4(),
                bet_value: 0,
                state: GameState::Running,
                moves: Vec::new(),
            })
        });

    mock_game_repository
        .expect_update_state()
        .withf(|id, g| {
            id == &uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b") && g == &GameState::BlackWin
        })
        .returning(|_, _| Ok(()));

    mock_rooms_manager
        .expect_remove_room()
        .once()
        .withf(|id| id == &uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"))
        .returning(|_| ());

    let service = DisconnectService::new(mock_game_repository, mock_rooms_manager);

    let result = service.execute(input).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_viewer_disconnection() {
    let mut mock_game_repository = MockGameRepositoryTrait::new();
    let mut mock_rooms_manager = MockRoomsManagerTrait::new();

    let input = DisconnectInfo {
        game_id: uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"),
        player_id: Uuid::new_v4(),
    };

    mock_rooms_manager
        .expect_get_room()
        .once()
        .withf(|id| id == &uuid::uuid!("73c1fad5-db48-4dce-8e03-6be3b43b0e7b"))
        .returning(|_| {
            Some(Room {
                request_key: String::from("w-10-0-0"),
                white_player: Some(Player {
                    id: Uuid::new_v4(),
                    ..Default::default()
                }),
                black_player: Some(Player {
                    id: Uuid::new_v4(),
                    ..Default::default()
                }),
                tx: broadcast::channel(100).0,
            })
        });

    mock_game_repository.expect_get_game().never();
    mock_game_repository.expect_update_state().never();
    mock_rooms_manager.expect_remove_room().never();

    let service = DisconnectService::new(mock_game_repository, mock_rooms_manager);

    let result = service.execute(input).await;

    assert!(result.is_ok());
}
