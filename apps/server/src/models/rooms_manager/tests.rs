use super::*;

#[test]
fn test_add_player_rooms_manager() {
    let rooms_manager = RoomsManager::_new_empty();

    let room_id = Uuid::new_v4();

    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();

    rooms_manager.create_room(room_id, String::from("123"));

    let player1_color = rooms_manager.add_player(room_id, player1, None);

    let room = rooms_manager.get_room(room_id).unwrap();

    assert!(room.white_player.is_some());
    assert!(room.black_player.is_none());
    assert!(player1_color == Ok(PlayerColor::White));
    assert_eq!(Some(player1), room.white_player);

    let player2_color = rooms_manager.add_player(room_id, player2, None);

    let room = rooms_manager.get_room(room_id).unwrap();

    assert!(room.white_player.is_some());
    assert!(room.black_player.is_some());
    assert!(player2_color == Ok(PlayerColor::Black));
    assert_eq!(Some(player2), room.black_player);
}

#[test]
fn test_add_player_to_room() {
    let mut room = Room::new(String::from("123"));

    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();

    let player1_color = room.add_player(player1, None);

    assert!(room.white_player.is_some());
    assert!(room.black_player.is_none());
    assert!(player1_color == Ok(PlayerColor::White));
    assert_eq!(Some(player1), room.white_player);

    let player2_color = room.add_player(player2, None);

    assert!(room.white_player.is_some());
    assert!(room.black_player.is_some());
    assert!(player2_color == Ok(PlayerColor::Black));
    assert_eq!(Some(player2), room.black_player);
}

#[test]
fn test_add_2_players_to_room() {
    let mut room = Room::new(String::from(""));

    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();

    let player1_color = room.add_player(player1, Some(PlayerColor::Black));

    assert!(player1_color.is_ok());
    assert!(room.black_player.is_some());
    assert!(room.white_player.is_none());
    assert_eq!(player1_color, Ok(PlayerColor::Black));

    let player2_color = room.add_player(player2, None);

    assert!(player2_color.is_ok());
    assert!(room.white_player.is_some());
    assert!(room.black_player.is_some());
    assert!(player2_color == Ok(PlayerColor::White));
}

#[test]
fn test_two_players_picking_white() {
    let mut room = Room::new(String::from(""));

    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();

    let player1_color = room.add_player(player1, Some(PlayerColor::White));

    assert!(player1_color.is_ok());
    assert!(room.white_player.is_some());
    assert!(room.black_player.is_none());
    assert_eq!(player1_color, Ok(PlayerColor::White));

    let player2_color = room.add_player(player2, Some(PlayerColor::White));

    assert!(player2_color.is_err());
    assert!(room.white_player.is_some());
    assert!(room.black_player.is_none());
}

#[test]
fn test_add_three_players() {
    let mut room = Room::new(String::from(""));

    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();
    let player3 = Uuid::new_v4();

    let player1_color = room.add_player(player1, Some(PlayerColor::White));

    assert!(player1_color.is_ok());
    assert!(room.white_player.is_some());
    assert!(room.black_player.is_none());
    assert_eq!(player1_color, Ok(PlayerColor::White));

    let player2_color = room.add_player(player2, None);

    assert!(player2_color.is_ok());
    assert!(room.white_player.is_some());
    assert!(room.black_player.is_some());
    assert_eq!(player2_color, Ok(PlayerColor::Black));

    let player3_color = room.add_player(player3, None);

    assert!(player3_color.is_err());
    assert!(room.white_player.is_some());
    assert!(room.black_player.is_some());
}
