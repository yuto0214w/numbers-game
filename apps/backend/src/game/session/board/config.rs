use numbers_comm_types::{Side, http::HttpBoardConfig};

use crate::game::MAX_TEAM_PLAYER_LIMIT;

#[derive(Debug, Clone)]
pub struct BoardConfig {
    pub team_player_limit: usize,
    pub first_side: Side,
}

impl TryFrom<HttpBoardConfig> for BoardConfig {
    type Error = &'static str;

    fn try_from(value: HttpBoardConfig) -> Result<Self, Self::Error> {
        let team_player_limit = match value.team_player_limit {
            Some(limit) if limit >= 1 && limit <= MAX_TEAM_PLAYER_LIMIT => limit,
            None => 1,
            _ => return Err("INVALID_PLAYER_LIMIT"),
        };
        Ok(Self {
            team_player_limit,
            first_side: value.first_side.unwrap_or(Side::A),
        })
    }
}
