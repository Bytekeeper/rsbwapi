use crate::{Game, Position};
use bwapi_wrapper::*;

#[derive(Clone, Copy)]
pub struct Region<'a> {
    game: &'a Game<'a>,
    data: &'a BWAPI_RegionData,
}

impl<'a> Region<'a> {
    pub(crate) fn new(game: &'a Game<'a>, data: &'a BWAPI_RegionData) -> Self {
        Self { game, data }
    }

    pub fn get_region_group_id(&self) -> i32 {
        self.data.islandID
    }

    pub fn get_center(&self) -> Position {
        Position {
            x: self.data.center_x,
            y: self.data.center_y,
        }
    }

    pub fn is_higher_ground(&self) -> bool {
        self.data.isHigherGround
    }

    pub fn get_defense_priority(&self) -> i32 {
        self.data.priority
    }

    pub fn is_accessible(&self) -> bool {
        self.data.isAccessible
    }

    pub fn get_id(&self) -> i32 {
        self.data.id
    }

    pub fn get_bounds_left(&self) -> i32 {
        self.data.leftMost
    }

    pub fn get_bounds_top(&self) -> i32 {
        self.data.topMost
    }

    pub fn get_bounds_right(&self) -> i32 {
        self.data.rightMost
    }

    pub fn get_bounds_bottom(&self) -> i32 {
        self.data.bottomMost
    }

    pub fn get_neighbors(&self) -> Vec<Region<'a>> {
        (0..self.data.neighborCount as usize)
            .map(|idx| {
                self.game
                    .get_region(idx as u16)
                    .expect("neighbor region to exist")
            })
            .collect()
    }

    pub fn get_closest_accessible_region(&self) -> Option<Region> {
        self.get_neighbors()
            .iter()
            .filter(|r| r.is_accessible())
            .min_by_key(|r| self.get_center().get_approx_distance(r.get_center()))
            .cloned()
    }

    pub fn get_closest_inaccessible_region(&self) -> Option<Region> {
        self.get_neighbors()
            .iter()
            .filter(|r| !r.is_accessible())
            .min_by_key(|r| self.get_center().get_approx_distance(r.get_center()))
            .cloned()
    }
}
