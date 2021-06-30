use super::{area::*, map::*, neutral::*};
use crate::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Base {
    p_map: Rc<RefCell<Map>>,
    p_area: Rc<RefCell<Area>>,
    location: TilePosition,
    center: Position,
    minerals: Vec<Rc<RefCell<Mineral>>>,
    geysers: Vec<Rc<RefCell<Geyser>>>,
    blocking_minerals: Vec<Rc<RefCell<Mineral>>>,
    starting: bool,
}
