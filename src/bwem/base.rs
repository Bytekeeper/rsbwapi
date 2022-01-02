use super::{area::*, map::*, neutral::*};
use crate::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Base {
    map: *const Map,
    area: *const Area,
    location: TilePosition,
    center: Position,
    minerals: Vec<*const Mineral>,
    geysers: Vec<*const Geyser>,
    blocking_minerals: Vec<*const Mineral>,
    starting: bool,
}

impl Base {
    pub fn new(
        area: *const Area,
        location: TilePosition,
        assigned_resources: Vec<*mut dyn Resource>,
        blocking_minerals: Vec<*const Mineral>,
    ) -> Self {
        let mut minerals = vec![];
        let mut geysers = vec![];
        for &r in assigned_resources.iter() {
            let r = unsafe { &*r };
            if let Some(mineral) = r.is_mineral() {
                minerals.push(mineral as *const Mineral);
            }
            if let Some(geyser) = r.is_geyser() {
                geysers.push(geyser as *const Geyser);
            }
        }
        Self {
            map: unsafe { &*area }.get_map(),
            area,
            location,
            center: location.to_position()
                + UnitType::Terran_Command_Center.tile_size().to_position() / 2,
            blocking_minerals,
            starting: false,
            geysers,
            minerals,
        }
    }

    pub fn location(&self) -> TilePosition {
        self.location
    }

    pub fn on_mineral_destroyed(&mut self, mineral: *mut Mineral) {
        unimplemented!();
    }
}
