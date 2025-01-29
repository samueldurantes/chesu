use super::*;
use crate::models::rooms_manager::PairedGame;
use crate::{
    models::{game::Player, game::PlayerColor, rooms_manager::MockRoomsManagerTrait},
    repositories::game_repository::MockGameRepositoryTrait,
};
use mockall::predicate::*;

#[tokio::test]
async fn quick_pairing_service() {
    let mut mock_game_repository = MockGameRepositoryTrait::new();
    let mut mock_rooms_manager = MockRoomsManagerTrait::new();

    let player_id = Uuid::new_v4();
    let game_id = Uuid::new_v4();
    let player = Player {
        id: player_id,
        username: String::from("romero"),
        email: String::from("romero@dias.com"),
    };
    let player_copy = player.clone();

    mock_game_repository
        .expect_get_player()
        .with(eq(player_id))
        .returning(move |_| Ok(player_copy.clone()));

    mock_game_repository
        .expect_save_game()
        .withf(move |game_record| {
            game_record.id == game_id && game_record.white_player == Some(player_id)
        })
        .returning(|_| Ok(()));

    mock_rooms_manager
        .expect_pair_new_player()
        .returning(move || PairedGame::NewGame(game_id));

    mock_rooms_manager
        .expect_create_room()
        .with(eq(game_id))
        .return_const(());

    mock_rooms_manager
        .expect_add_player()
        .with(
            eq(game_id),
            eq(player.clone()),
            eq(Some(PlayerColor::White)),
        )
        .returning(|_, _, _| Ok(PlayerColor::White));

    let service = PairingGameService::new(mock_game_repository, mock_rooms_manager);

    let result = service.execute(player_id).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), game_id);
}
