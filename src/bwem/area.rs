use super::{base::*, cp::*, defs::*, graph::Graph, neutral::*};
use crate::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type AreaId = i16;
pub type GroupId = i16;

pub struct Area {
    p_graph: Rc<RefCell<Graph>>,
    id: AreaId,
    group_id: GroupId,
    top: WalkPosition,
    top_left: TilePosition,
    bottom_right: TilePosition,
    max_altitude: Altitude,
    mini_tiles: isize,
    tiles: isize,
    buildable_tiles: isize,
    high_ground_tiles: isize,
    very_high_ground_tiles: isize,
    choke_points_by_area: HashMap<Rc<RefCell<Area>>, Rc<RefCell<Vec<ChokePoint>>>>,
    accessible_neighbours: Vec<Rc<RefCell<Area>>>,
    choke_points: Vec<Rc<RefCell<ChokePoint>>>,
    minerals: Vec<Rc<RefCell<Mineral>>>,
    geyser: Vec<Rc<RefCell<Geyser>>>,
    base: Vec<Rc<RefCell<Base>>>,
}
