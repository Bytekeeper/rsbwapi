use super::{area::*, graph::*, neutral::*};
use crate::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

pub type Index = isize;

pub struct ChokePoint {
    p_graph: Rc<RefCell<Graph>>,
    pseudo: bool,
    index: Index,
    areas: (Rc<RefCell<Area>>, Rc<RefCell<Area>>),
    nodes: [WalkPosition; 4],
    nodes_in_area: [(WalkPosition, WalkPosition); 4],
    geometry: VecDeque<WalkPosition>,
    blocked: bool,
    blocking_neutral: Option<Rc<RefCell<Neutral>>>,
    p_path_back_trace: Option<Rc<RefCell<ChokePoint>>>,
}
