use crate::projected::Projected;
use crate::{Game, Position};
use bwapi_wrapper::*;

#[derive(Clone)]
pub struct Region {
    inner: Projected<Game, BWAPI_RegionData>,
}

impl Region {
    pub(crate) fn new(id: u16, game: Game) -> Self {
        let region = &game.inner.data.regions[id as usize] as *const BWAPI_RegionData;
        Self {
            inner: unsafe { Projected::new(game, region) },
        }
    }

    pub fn get_region_group_id(&self) -> i32 {
        self.inner.islandID
    }

    pub fn get_center(&self) -> Position {
        Position {
            x: self.inner.center_x,
            y: self.inner.center_y,
        }
    }

    pub fn is_higher_ground(&self) -> bool {
        self.inner.isHigherGround
    }

    pub fn get_defense_priority(&self) -> i32 {
        self.inner.priority
    }

    pub fn is_accessible(&self) -> bool {
        self.inner.isAccessible
    }

    pub fn get_id(&self) -> i32 {
        self.inner.id
    }

    pub fn get_bounds_left(&self) -> i32 {
        self.inner.leftMost
    }

    pub fn get_bounds_top(&self) -> i32 {
        self.inner.topMost
    }

    pub fn get_bounds_right(&self) -> i32 {
        self.inner.rightMost
    }

    pub fn get_bounds_bottom(&self) -> i32 {
        self.inner.bottomMost
    }

    pub fn get_neighbors(&self) -> Vec<Region> {
        (0..self.inner.neighborCount as usize)
            .map(|idx| {
                self.inner
                    .game()
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
