use super::{area::*, graph::*, map::*, neutral::*};
use crate::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

pub type Index = isize;

///////////////////////////////////////////////////////////////////////////////////////////////
///                                                                                          //
/// class ChokePoint
///                                                                                          //
///////////////////////////////////////////////////////////////////////////////////////////////
///
/// ChokePoints are frontiers that BWEM automatically computes from Brood War's maps
/// A ChokePoint represents (part of) the frontier between exactly 2 Areas. It has a form of line.
/// A ChokePoint doesn't contain any MiniTile: All the MiniTiles whose positions are returned by its Geometry()
/// are just guaranteed to be part of one of the 2 Areas.
/// Among the MiniTiles of its Geometry, 3 particular ones called nodes can also be accessed using Pos(middle), Pos(end1) and Pos(end2).
/// ChokePoints play an important role in BWEM:
///   - they define accessibility between Areas.
///   - the Paths provided by Map::GetPath are made of ChokePoints.
/// Like Areas and Bases, the number and the addresses of ChokePoint instances remain unchanged.
///
/// Pseudo ChokePoints:
/// Some Neutrals can be detected as blocking Neutrals (Cf. Neutral::Blocking).
/// Because only ChokePoints can serve as frontiers between Areas, BWEM automatically creates a ChokePoint
/// for each blocking Neutral (only one in the case of stacked blocking Neutral).
/// Such ChokePoints are called pseudo ChokePoints and they behave differently in several ways.
///
/// ChokePoints inherit utils::Markable, which provides marking ability
/// ChokePoints inherit utils::UserData, which provides free-to-use data.
pub struct ChokePoint {
    p_graph: Rc<RefCell<Graph>>,
    pseudo: bool,
    index: Index,
    areas: (Rc<RefCell<Area>>, Rc<RefCell<Area>>),
    nodes: [WalkPosition; 4],
    nodes_in_area: [(WalkPosition, WalkPosition); 4],
    geometry: VecDeque<WalkPosition>,
    blocked: bool,
    blocking_neutral: Option<Rc<dyn Neutral>>,
    p_path_back_trace: Option<Rc<RefCell<ChokePoint>>>,
}

/// ChokePoint::middle denotes the "middle" MiniTile of Geometry(), while
/// ChokePoint::end1 and ChokePoint::end2 denote its "ends".
/// It is guaranteed that, among all the MiniTiles of Geometry(), ChokePoint::middle has the highest altitude value (Cf. MiniTile::Altitude()).
pub enum Node {
    End1,
    Middle,
    End2,
    NodeCount,
}

/// Type of all the Paths used in BWEM (Cf. Map::GetPath).
/// See also the typedef CPPath.
pub type Path = Vec<Rc<RefCell<ChokePoint>>>;

impl ChokePoint {
    /// Tells whether this ChokePoint is a pseudo ChokePoint, i.e., it was created on top of a blocking Neutral.
    pub fn is_pseudo(&self) -> bool {
        self.pseudo
    }

    /// Returns the two Areas of this ChokePoint.
    pub fn get_areas(&self) -> (Rc<RefCell<Area>>, Rc<RefCell<Area>>) {
        self.areas.clone()
    }

    /// Returns the center of this ChokePoint.
    pub fn center(&self) -> WalkPosition {
        self.pos(Node::Middle)
    }

    /// Returns the position of one of the 3 nodes of this ChokePoint (Cf. node definition).
    /// Note: the returned value is contained in Geometry()
    pub fn pos(&self, n: Node) -> WalkPosition {
        match n {
            Node::End1 => self.nodes[0],
            Node::Middle => self.nodes[1],
            Node::End2 => self.nodes[2],
            _ => panic!(),
        }
    }

    /// Pretty much the same as Pos(n), except that the returned MiniTile position is guaranteed to be part of pArea.
    /// That is: Map::GetArea(PosInArea(n, pArea)) == pArea.
    pub fn pos_in_area(&self, n: Node, area: &mut Area) -> TilePosition {
        unimplemented!();
    }

    /// Returns the set of positions that defines the shape of this ChokePoint.
    /// Note: none of these MiniTiles actually belongs to this ChokePoint (a ChokePoint doesn't contain any MiniTile).
    ///       They are however guaranteed to be part of one of the 2 Areas.
    /// Note: the returned set contains Pos(middle), Pos(end1) and Pos(end2).
    /// If IsPseudo(), returns {p} where p is the position of a walkable MiniTile near from BlockingNeutral()->Pos().
    pub fn geometry(&self) -> &VecDeque<WalkPosition> {
        &self.geometry
    }

    /// If !IsPseudo(), returns false.
    /// Otherwise, returns whether this ChokePoint is considered blocked.
    /// Normally, a pseudo ChokePoint either remains blocked, or switches to not blocked when BlockingNeutral()
    /// is destroyed and there is no remaining Neutral stacked with it.
    /// However, in the case where Map::AutomaticPathUpdate() == false, Blocked() will always return true
    /// whatever BlockingNeutral() returns.
    /// Cf. Area::AccessibleNeighbours().
    pub fn blocked(&self) -> bool {
        self.blocked
    }

    /// If !IsPseudo(), returns nullptr.
    /// Otherwise, returns a pointer to the blocking Neutral on top of which this pseudo ChokePoint was created,
    /// unless this blocking Neutral has been destroyed.
    /// In this case, returns a pointer to the next blocking Neutral that was stacked at the same location,
    /// or nullptr if no such Neutral exists.
    pub fn blocking_neutral(&self) -> Option<Rc<dyn Neutral>> {
        self.blocking_neutral.clone()
    }

    /// If AccessibleFrom(cp) == false, returns -1.
    /// Otherwise, returns the ground distance in pixels between Center() and cp->Center().
    /// Note: if this == cp, returns 0.
    /// Time complexity: O(1)
    /// Note: Corresponds to the length in pixels of GetPathTo(cp). So it suffers from the same lack of accuracy.
    ///       In particular, the value returned tends to be slightly higher than expected when GetPathTo(cp).size() is high.
    pub fn distance_from(&self, cp: Rc<RefCell<ChokePoint>>) -> usize {
        unimplemented!()
    }

    /// Returns whether this ChokePoint is accessible from cp (through a walkable path).
    /// Note: the relation is symmetric: this->AccessibleFrom(cp) == cp->AccessibleFrom(this)
    /// Note: if this == cp, returns true.
    /// Time complexity: O(1)
    pub fn accessible_from(&self, cp: Rc<RefCell<ChokePoint>>) -> bool {
        self.distance_from(cp) >= 0
    }

    /// Returns a list of ChokePoints, which is intended to be the shortest walking path from this ChokePoint to cp.
    /// The path always starts with this ChokePoint and ends with cp, unless AccessibleFrom(cp) == false.
    /// In this case, an empty list is returned.
    /// Note: if this == cp, returns [cp].
    /// Time complexity: O(1)
    /// To get the length of the path returned in pixels, use DistanceFrom(cp).
    /// Note: all the possible Paths are precomputed during Map::Initialize().
    ///       The best one is then stored for each pair of ChokePoints.
    ///       However, only the center of the ChokePoints is considered.
    ///       As a consequence, the returned path may not be the shortest one.
    pub fn get_path_to(&self, cp: Rc<RefCell<ChokePoint>>) -> &Path {
        unimplemented!()
    }

    pub fn get_map(&self) -> Rc<RefCell<Map>> {
        unimplemented!()
    }

    //    ChokePoint &                                                    operator=(const ChokePoint &) = delete;

    ////////////////////////////////////////////////////////////////////////////
    //      Details: The functions below are used by the BWEM's internals

    pub fn new(
        graph: Rc<RefCell<Graph>>,
        idx: Index,
        area1: Rc<RefCell<Area>>,
        area2: Rc<RefCell<Area>>,
        geometry: VecDeque<WalkPosition>,
        neutral: Option<Rc<dyn Neutral>>,
    ) {
        unimplemented!();
    }

    pub fn on_blocking_neutral_destroyed(&mut self, neutral: Rc<dyn Neutral>) {
        unimplemented!();
    }

    pub fn index(&self) -> Index {
        self.index
    }

    pub fn path_back_trace(&self) -> Option<Rc<RefCell<ChokePoint>>> {
        self.p_path_back_trace.clone()
    }

    pub fn set_path_back_trace(&mut self, p: Rc<RefCell<ChokePoint>>) {
        self.p_path_back_trace = Some(p);
    }

    fn get_graph(&self) -> Rc<RefCell<Graph>> {
        self.p_graph.clone()
    }
}
