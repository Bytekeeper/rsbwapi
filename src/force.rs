use crate::types::c_str_to_str;
use crate::Player;
use bwapi_wrapper::*;

pub struct Force {
    pub id: usize,
    pub name: String,
    pub players: Vec<Player>,
}

impl Force {
    pub fn new(id: usize, data: &BWAPI_ForceData, players: Vec<Player>) -> Self {
        Self {
            id,
            name: c_str_to_str(&data.name),
            players,
        }
    }

    pub fn get_players(&self) -> &[Player] {
        &self.players
    }
}
