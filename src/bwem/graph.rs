use super::{area::*, cp::*, map::*};
use std::cell::RefCell;
use std::rc::Rc;

const empty_path: Path = vec![];

pub struct Graph {
    p_map: Rc<RefCell<Map>>,
    areas: Vec<Area>,
    choke_point_list: Vec<Rc<RefCell<ChokePoint>>>,
    choke_points_matrix: Vec<Vec<Vec<ChokePoint>>>,
    choke_point_distance_matrix: Vec<Vec<isize>>,
    paths_between_choke_points: Vec<Vec<Path>>,
}
