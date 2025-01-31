use super::*;
use crate::models::rooms_manager::PairedGame;
use crate::{
    models::{game::Player, game::PlayerColor, rooms_manager::MockRoomsManagerTrait},
    repositories::game_repository::MockGameRepositoryTrait,
};
use mockall::predicate::*;
use uuid::uuid;

#[tokio::test]
async fn quick_pairing_service() {
    let mut mock_game_repository = MockGameRepositoryTrait::new();
    let mut mock_rooms_manager = MockRoomsManagerTrait::new();

    let request_key = "w-10-0-0";
    let player = Player {
        id: uuid!("5d6cc3e8-8eec-4dab-881f-fddfb831cc41"),
        ..Default::default()
    };

    mock_rooms_manager
        .expect_pair_new_player()
        .returning(|_| PairedGame::NewGame(uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a")));

    mock_game_repository
        .expect_get_player()
        .with(eq(player.id))
        .returning(|p_id| {
            Ok(Player {
                id: p_id,
                ..Default::default()
            })
        });

    mock_rooms_manager
        .expect_create_room()
        .with(
            eq(uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a")),
            eq(request_key.to_string()),
        )
        .return_const(());

    mock_rooms_manager
        .expect_add_player()
        .returning(|_, _, c| Ok(c.unwrap_or(PlayerColor::White)));

    let service = PairingGameService::new(mock_game_repository, mock_rooms_manager);

    let game_request = GameRequest::from_str(request_key);

    assert!(game_request.is_ok());

    let result = service.execute(player.id, game_request.unwrap()).await;

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        uuid!("06d6a0d9-97a8-48d0-9f81-0172c5a81b8a")
    );
}

#[test]
fn test_request_key_parsing_1() {
    let input = "";
    let result = GameRequest::from_str(input);

    assert!(result.is_err());
}

#[test]
fn test_request_key_parsing_2() {
    let input = "w-10-0-0";
    let result = GameRequest::from_str(input);

    assert!(result.is_ok());
    assert_eq!(
        result,
        Ok(GameRequest {
            key: input.to_string(),
            player_color: Some(PlayerColor::White),
            _total_time: 10,
            _turn_time: 0,
            bet_value: 0,
        })
    )
}

#[test]
fn test_request_key_parsing_3() {
    let input = "j-10-0-0";
    let result = GameRequest::from_str(input);

    assert!(result.is_err());
}

#[test]
fn test_request_key_parsing_4() {
    let input = "b-30-10-10000";
    let result = GameRequest::from_str(input);

    assert!(result.is_ok());
    assert_eq!(
        result,
        Ok(GameRequest {
            key: input.to_string(),
            player_color: Some(PlayerColor::Black),
            _total_time: 30,
            _turn_time: 10,
            bet_value: 10000,
        })
    )
}
