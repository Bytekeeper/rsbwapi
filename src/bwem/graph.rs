use super::{area::*, cp::*, map::*};
use crate::*;
use std::cell::RefCell;
use std::rc::Rc;

const empty_path: Path = vec![];

pub struct Graph {
    p_map: *mut Map,
    areas: Vec<Area>,
    choke_point_list: Vec<Rc<RefCell<ChokePoint>>>,
    choke_points_matrix: Vec<Vec<Vec<ChokePoint>>>,
    choke_point_distance_matrix: Vec<Vec<isize>>,
    paths_between_choke_points: Vec<Vec<Path>>,
}

impl Graph {
    pub fn get_map(&self) -> *mut Map {
        self.p_map
    }

    pub fn create_areas(&mut self, areas_list: Vec<(WalkPosition, i32)>) {
        unimplemented!()
    }
}
