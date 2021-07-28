use super::{base::*, cp::*, defs::*, graph::Graph, map::*, neutral::*, tiles::*};
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
    geysers: Vec<Rc<RefCell<Geyser>>>,
    bases: Vec<Base>,
}

impl Area {
    /// Unique id > 0 of this Area. Range = 1 .. Map::Areas().size()
    /// this == Map::GetArea(Id())
    /// Id() == Map::GetMiniTile(w).AreaId() for each walkable MiniTile w in this Area.
    /// Area::ids are guaranteed to remain unchanged.
    pub fn id(&self) -> AreaId {
        self.id
    }

    /// Unique id > 0 of the group of Areas which are accessible from this Area.
    /// For each pair (a, b) of Areas: a->GroupId() == b->GroupId()  <==>  a->AccessibleFrom(b)
    /// A groupId uniquely identifies a maximum set of mutually accessible Areas, that is, in the absence of blocking ChokePoints, a continent.
    pub fn group_id(&self) -> GroupId {
        self.group_id
    }

    /// Bounding box of this Area.
    pub fn top_left(&self) -> TilePosition {
        self.top_left
    }

    pub fn bottom_right(&self) -> TilePosition {
        self.bottom_right
    }

    pub fn bounding_box_size(&self) -> TilePosition {
        unimplemented!()
    }

    /// Position of the MiniTile with the highest Altitude() value.
    pub fn top(&self) -> WalkPosition {
        self.top
    }

    /// Returns Map::GetMiniTile(Top()).Altitude().
    pub fn max_altitude(&self) -> Altitude {
        self.max_altitude
    }

    /// Returns the number of MiniTiles in this Area.
    /// This most accurately defines the size of this Area.
    pub fn mini_tiles(&self) -> isize {
        self.mini_tiles
    }

    /// Returns the percentage of low ground Tiles in this Area.
    pub fn low_ground_percentage(&self) -> isize {
        (self.tiles - self.high_ground_tiles - self.very_high_ground_tiles) * 100 / self.tiles
    }

    /// Returns the percentage of high ground Tiles in this Area.
    pub fn high_ground_percentage(&self) -> isize {
        self.high_ground_tiles * 100 / self.tiles
    }

    /// Returns the percentage of very high ground Tiles in this Area.
    pub fn very_high_ground_percentage(&self) -> isize {
        self.very_high_ground_tiles * 100 / self.tiles
    }

    /// Returns the ChokePoints between this Area and the neighbouring ones.
    /// Note: if there are no neighbouring Areas, then an empty set is returned.
    /// Note there may be more ChokePoints returned than the number of neighbouring Areas, as there may be several ChokePoints between two Areas (Cf. ChokePoints(const Area * pArea)).
    pub fn choke_points(&self) -> Vec<Rc<RefCell<ChokePoint>>> {
        self.choke_points.clone()
    }

    /// Returns the ChokePoints of this Area grouped by neighbouring Areas
    /// Note: if there are no neighbouring Areas, than an empty set is returned.
    pub fn choke_points_by_area(&self) -> HashMap<Rc<RefCell<Area>>, Rc<RefCell<Vec<ChokePoint>>>> {
        self.choke_points_by_area.clone()
    }

    /// Returns the accessible neighbouring Areas.
    /// The accessible neighbouring Areas are a subset of the neighbouring Areas (the neighbouring Areas can be iterated using ChokePointsByArea()).
    /// Two neighbouring Areas are accessible from each over if at least one the ChokePoints they share is not Blocked (Cf. ChokePoint::Blocked).
    pub fn accessible_neighbours(&self) -> &[Rc<RefCell<Area>>] {
        &self.accessible_neighbours
    }

    /// Returns whether this Area is accessible from pArea, that is, if they share the same GroupId().
    /// Note: accessibility is always symmetrical.
    /// Note: even if a and b are neighbouring Areas,
    ///       we can have: a->AccessibleFrom(b)
    ///       and not:     contains(a->AccessibleNeighbours(), b)
    /// See also GroupId()
    pub fn accessible_from(&self, area: Rc<RefCell<Area>>) -> bool {
        self.group_id() == area.borrow().group_id()
    }

    /// Returns the Minerals contained in this Area.
    /// Note: only a call to Map::OnMineralDestroyed(BWAPI::Unit u) may change the result (by removing eventually one element).
    pub fn mineral(&self) -> Vec<Rc<RefCell<Mineral>>> {
        self.minerals.clone()
    }

    /// Returns the Geysers contained in this Area.
    /// Note: the result will remain unchanged.
    pub fn geysers(&self) -> Vec<Rc<RefCell<Geyser>>> {
        self.geysers.clone()
    }

    /// Returns the Bases contained in this Area.
    /// Note: the result will remain unchanged.
    pub fn bases(&self) -> &[Base] {
        &self.bases
    }

    pub fn get_map(&self) -> Rc<RefCell<Map>> {
        unimplemented!()
    }

    pub fn add_choke_points(&mut self, area: Rc<RefCell<Area>>, choke_points: &[ChokePoint]) {
        unimplemented!()
    }

    pub fn add_mineral(&mut self, mineral: Rc<RefCell<Mineral>>) {
        unimplemented!()
    }

    pub fn add_geyser(&mut self, geyser: Rc<RefCell<Geyser>>) {
        unimplemented!()
    }

    pub fn add_tile_information(&mut self, t: TilePosition, tile: Rc<RefCell<Tile>>) {
        unimplemented!()
    }

    pub fn on_mineral_destroyed(&mut self, mineral: Rc<RefCell<Mineral>>) {
        unimplemented!()
    }

    pub fn post_collect_information(&self) {
        unimplemented!()
    }

    pub fn compute_distances(
        &mut self,
        start_cp: Rc<RefCell<ChokePoint>>,
        target_cps: &[Rc<RefCell<ChokePoint>>],
    ) -> Vec<isize> {
        unimplemented!()
    }

    pub fn update_accessible_neighbours(&mut self) {
        unimplemented!()
    }

    pub fn set_group_id(&mut self, gid: GroupId) {
        debug_assert!(gid >= 1);
        self.group_id = gid;
    }

    pub fn create_bases(&mut self) {
        unimplemented!()
    }

    fn get_graph(&self) -> Rc<RefCell<Graph>> {
        self.p_graph.clone()
    }

    fn compute_base_location_score(&self, location: TilePosition) -> isize {
        unimplemented!()
    }

    fn validate_base_location(
        &self,
        location: TilePosition,
        blocking_minerals: &[Rc<RefCell<Mineral>>],
    ) -> bool {
        unimplemented!()
    }

    fn compute_distances_starting_at(
        &self,
        start: TilePosition,
        targets: &[TilePosition],
    ) -> Vec<isize> {
        unimplemented!()
    }
}
