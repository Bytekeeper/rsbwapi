use crate::types::c_str_to_str;
use crate::Player;
use bwapi_wrapper::*;

pub struct Force<'a> {
    pub id: usize,
    pub name: String,
    pub players: Vec<Player<'a>>,
}

impl<'a> Force<'a> {
    pub fn new(id: usize, data: &BWAPI_ForceData, players: Vec<Player<'a>>) -> Self {
        Self {
            id,
            name: c_str_to_str(&data.name).to_owned(),
            players,
        }
    }

    pub fn get_players(&self) -> &Vec<Player<'a>> {
        &self.players
    }
}
