use super::map::*;
use crate::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct UnitMemo {
    id: UnitId,
}

pub struct Neutral {
    bwapi_unit: UnitMemo,
    bwapi_type: UnitType,
    pos: Position,
    top_left: TilePosition,
    size: TilePosition,
    p_map: Rc<RefCell<Map>>,
    p_next_stacked: Option<Rc<RefCell<Neutral>>>,
    blocked_areas: Vec<WalkPosition>,
}

impl PartialEq for Neutral {
    fn eq(&self, other: &Self) -> bool {
        self.bwapi_unit.id == other.bwapi_unit.id
    }
}

pub struct Resource {
    initial_amount: isize,
}

pub struct Mineral {}

pub struct Geyser {}

pub struct StaticBuilding {}
