use super::game::PlayerColor;
use crate::http::{Error, Result};

#[derive(Debug, PartialEq)]
pub struct GameRequest {
    pub key: String,
    pub player_color: Option<PlayerColor>,
    pub total_time: u8,
    pub turn_time: u8,
    pub bet_value: i32,
}

fn invalid_game_request() -> Error {
    Error::BadRequest {
        message: String::from("Invalid game request!"),
    }
}

fn resolve_i32(input: Option<&str>) -> Result<i32> {
    input
        .ok_or(invalid_game_request())?
        .parse::<i32>()
        .map_err(|_| invalid_game_request())
}

fn resolve_u8(input: Option<&str>) -> Result<u8> {
    input
        .ok_or(invalid_game_request())?
        .parse::<u8>()
        .map_err(|_| invalid_game_request())
}

fn resolve_player_color(input: Option<&str>) -> Result<Option<PlayerColor>> {
    let input = input.ok_or(invalid_game_request())?;

    match input {
        "w" => Ok(Some(PlayerColor::White)),
        "b" => Ok(Some(PlayerColor::Black)),
        "n" => Ok(None),
        _ => Err(invalid_game_request()),
    }
}

impl GameRequest {
    pub fn from_str(key: &str) -> Result<Self> {
        let mut result = key.split("-");

        let player_color = resolve_player_color(result.next())?;
        let total_time = resolve_u8(result.next())?;
        let turn_time = resolve_u8(result.next())?;
        let bet_value = resolve_i32(result.next())?;

        if total_time <= 0 || bet_value < 0 {
            return Err(invalid_game_request());
        }

        Ok(Self {
            key: key.to_string(),
            total_time,
            turn_time,
            bet_value,
            player_color,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::models::game::PlayerColor;

    use super::*;

    #[test]
    fn test_request_key_parsing_1() {
        let input = "";
        let result = GameRequest::from_str(input);

        assert!(result.is_err());
    }

    #[test]
    fn test_request_key_parsing_2() {
        let input = "w-10-0-0";
        let result = GameRequest::from_str(input).ok();

        assert!(result.is_some());
        assert_eq!(
            result,
            Some(GameRequest {
                key: input.to_string(),
                player_color: Some(PlayerColor::White),
                total_time: 10,
                turn_time: 0,
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
        let result = GameRequest::from_str(input).ok();

        assert!(result.is_some());
        assert_eq!(
            result,
            Some(GameRequest {
                key: input.to_string(),
                player_color: Some(PlayerColor::Black),
                total_time: 30,
                turn_time: 10,
                bet_value: 10000,
            })
        )
    }
}
