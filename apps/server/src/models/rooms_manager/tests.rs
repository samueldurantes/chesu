use super::*;

#[test]
fn test_add_player_rooms_manager() {
    let rooms_manager = RoomsManager::_new_empty();

    let room_id = Uuid::new_v4();

    let player1 = Player {
        id: Uuid::new_v4(),
        username: String::from("Ding Liren"),
        email: String::from("ding@liren.com"),
    };

    let player2 = Player {
        id: Uuid::new_v4(),
        username: String::from("Elon Musk"),
        email: String::from("elon@musk.com"),
    };

    let p1_id = player1.id.clone();
    let p2_id = player2.id.clone();

    rooms_manager.create_room(room_id);

    let player1_color = rooms_manager.add_player(room_id, player1, None).unwrap();

    let room = rooms_manager.get_room(room_id).unwrap();

    assert!(room.white_player.is_some());
    assert!(room.black_player.is_none());
    assert!(player1_color == PlayerColor::White);
    assert_eq!(p1_id, room.white_player.clone().unwrap().id);

    let player2_color = rooms_manager.add_player(room_id, player2, None).unwrap();

    let room = rooms_manager.get_room(room_id).unwrap();

    assert!(room.white_player.is_some());
    assert!(room.black_player.is_some());
    assert!(player2_color == PlayerColor::Black);
    assert_eq!(p2_id, room.black_player.unwrap().id);
}

#[test]
fn test_add_player_to_room() {
    let mut room = Room::new();

    let player1 = Player {
        id: Uuid::new_v4(),
        username: String::from("Ding Liren"),
        email: String::from("ding@liren.com"),
    };

    let player2 = Player {
        id: Uuid::new_v4(),
        username: String::from("Ding Liren"),
        email: String::from("ding@liren.com"),
    };

    let p1_id = player1.id.clone();
    let p2_id = player2.id.clone();

    let player1_color = room.add_player(player1, None).unwrap();

    assert!(room.white_player.is_some());
    assert!(room.black_player.is_none());
    assert!(player1_color == PlayerColor::White);
    assert_eq!(p1_id, room.white_player.clone().unwrap().id);

    let player2_color = room.add_player(player2, None).unwrap();

    assert!(room.white_player.is_some());
    assert!(room.black_player.is_some());
    assert!(player2_color == PlayerColor::Black);
    assert_eq!(p2_id, room.black_player.unwrap().id);
}
