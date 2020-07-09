use crate::types::c_str_to_str;
use crate::types::UnitType;
use bwapi_wrapper::*;

#[derive(Clone, Copy)]
pub struct Player<'a> {
    pub id: usize,
    data: &'a BWAPI_PlayerData,
}

impl<'a> Player<'a> {
    pub(crate) fn new(id: usize, data: &'a BWAPI_PlayerData) -> Self {
        Player { id, data }
    }

    pub fn name(&self) -> &str {
        c_str_to_str(&self.data.name)
    }

    pub fn armor(&self, _unit_type: UnitType) -> i32 {
        unimplemented!()
    }

    pub fn is_ally(&self, other: &Player) -> bool {
        self.data.isAlly[other.id]
    }

    pub fn is_observer(&self) -> bool {
        !self.data.isParticipating
    }
}

impl<'a> PartialEq for Player<'a> {
    fn eq(&self, other: &Player) -> bool {
        self.id == other.id
    }
}
