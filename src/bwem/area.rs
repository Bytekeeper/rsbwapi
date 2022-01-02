use super::{base::*, cp::*, defs::*, graph::Graph, map::*, neutral::*, tiles::*};
use crate::{bwem::*, *};
use ahash::AHashMap;
use std::cell::RefCell;
use std::collections::{BinaryHeap, HashMap};
use std::rc::Rc;

pub type AreaId = i16;
pub type GroupId = i16;

pub struct Area {
    p_graph: *mut Graph,
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
    choke_points_by_area: AHashMap<*const Area, *const Vec<ChokePoint>>,
    accessible_neighbours: Vec<*const Area>,
    choke_points: Vec<*const ChokePoint>,
    minerals: Vec<*mut Mineral>,
    geysers: Vec<*mut Geyser>,
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
        self.bottom_right - self.top_left + (1, 1)
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
    pub fn choke_points(&self) -> &[*const ChokePoint] {
        &self.choke_points
    }

    /// Returns the ChokePoints of this Area grouped by neighbouring Areas
    /// Note: if there are no neighbouring Areas, than an empty set is returned.
    pub fn choke_points_by_area(&self) -> &AHashMap<*const Area, *const Vec<ChokePoint>> {
        &self.choke_points_by_area
    }

    /// Returns the accessible neighbouring Areas.
    /// The accessible neighbouring Areas are a subset of the neighbouring Areas (the neighbouring Areas can be iterated using ChokePointsByArea()).
    /// Two neighbouring Areas are accessible from each over if at least one the ChokePoints they share is not Blocked (Cf. ChokePoint::Blocked).
    pub fn accessible_neighbours(&self) -> &[*const Area] {
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
    pub fn minerals(&self) -> &[*mut Mineral] {
        &self.minerals
    }

    /// Returns the Geysers contained in this Area.
    /// Note: the result will remain unchanged.
    pub fn geysers(&self) -> &[*mut Geyser] {
        &self.geysers
    }

    /// Returns the Bases contained in this Area.
    /// Note: the result will remain unchanged.
    pub fn bases(&self) -> &[Base] {
        &self.bases
    }

    pub fn get_map(&self) -> *mut Map {
        unsafe { self.p_graph.as_ref().unwrap() }.get_map()
    }

    pub fn add_choke_points(&mut self, area: *const Area, choke_points: *const Vec<ChokePoint>) {
        debug_assert!(!self.choke_points_by_area.contains_key(&area) && !choke_points.is_null());

        self.choke_points_by_area.insert(area, choke_points);
        for cp in unsafe { choke_points.as_ref().unwrap() }.iter() {
            self.choke_points.push(cp);
        }
    }

    pub fn add_mineral(&mut self, mineral: *mut Mineral) {
        debug_assert!(!mineral.is_null() && !self.minerals.contains(&mineral));

        self.minerals.push(mineral);
    }

    pub fn add_geyser(&mut self, geyser: *mut Geyser) {
        debug_assert!(!geyser.is_null() && !self.geysers.contains(&geyser));

        self.geysers.push(geyser);
    }

    pub fn add_tile_information(&mut self, t: TilePosition, tile: &Tile) {
        self.tiles += 1;
        if tile.buildable() {
            self.buildable_tiles += 1
        }
        if tile.ground_height() == 1 {
            self.buildable_tiles += 1
        }
        if tile.ground_height() == 2 {
            self.very_high_ground_tiles += 1
        }

        self.top_left.x = self.top_left.x.min(t.x);
        self.top_left.y = self.top_left.y.min(t.y);
        self.bottom_right.x = self.bottom_right.x.max(t.x);
        self.bottom_right.y = self.bottom_right.y.max(t.y);
    }

    pub fn on_mineral_destroyed(&mut self, mineral: *mut Mineral) {
        debug_assert!(!mineral.is_null());

        if let Some(i) = self.minerals.iter().position(|&m| m == mineral) {
            self.minerals.swap_remove(i);
        }

        // let's examine the bases even if pMineral was not found in this Area,
        // which could arise if Minerals were allowed to be assigned to neighbouring Areas.
        for base in self.bases.iter_mut() {
            base.on_mineral_destroyed(mineral);
        }
    }

    pub fn post_collect_information(&self) {}

    pub fn compute_distances(
        &mut self,
        start_cp: *const ChokePoint,
        target_cps: &Vec<*const ChokePoint>,
    ) -> Vec<i32> {
        debug_assert!(!target_cps.contains(&start_cp));
        let map = unsafe { &*self.get_map() };

        let start = map.breadth_first_search(
            unsafe { &*start_cp }.pos_in_area(Node::Middle, self),
            |tile: &Tile, _: TilePosition| tile.area_id() == self.id(),
            |_: &Tile, _: TilePosition| true,
            true,
        );
        let targets: Vec<_> = target_cps
            .iter()
            .map(|&cp| {
                map.breadth_first_search(
                    unsafe { &*cp }.pos_in_area(Node::Middle, self),
                    |tile: &Tile, _: TilePosition| tile.area_id() == self.id(),
                    |_: &Tile, _: TilePosition| true,
                    true,
                )
            })
            .collect();
        self.compute_distances_starting_at(start, &targets)
    }

    pub fn update_accessible_neighbours(&mut self) {
        self.accessible_neighbours = self
            .choke_points_by_area()
            .iter()
            .filter(|(k, &v)| unsafe { &*v }.iter().any(|cp| !cp.blocked()))
            .map(|(k, _)| *k)
            .collect();
    }

    pub fn set_group_id(&mut self, gid: GroupId) {
        debug_assert!(gid >= 1);
        self.group_id = gid;
    }

    pub fn create_bases(&mut self) {
        let dim_cc = UnitType::Terran_Command_Center.tile_size();

        let map = unsafe { &mut *self.get_map() };

        // Initialize the RemainingRessources with all the Minerals and Geysers in this Area satisfying some conditions:

        let mut remaining_resources: Vec<*mut dyn Resource> = self
            .minerals()
            .iter()
            .filter(|&m| {
                let m = unsafe { &**m };
                m.initial_amount() >= 40 && !m.blocking()
            })
            .map(|&m| m as *mut dyn Resource)
            .chain(
                self.geysers()
                    .iter()
                    .filter(|&g| {
                        let g = unsafe { &**g };
                        g.initial_amount() >= 300 && !g.blocking()
                    })
                    .map(|&g| g as *mut dyn Resource),
            )
            .collect();

        while !remaining_resources.is_empty() {
            // 1) Calculate the SearchBoundingBox (needless to search too far from the RemainingRessources):

            let mut top_left_resources = TilePosition::new(std::i32::MAX, std::i32::MAX);
            let mut bottom_right_resources = TilePosition::new(std::i32::MIN, std::i32::MIN);
            for &r in remaining_resources.iter() {
                let r = unsafe { &*r };
                make_bounding_box_include_point(
                    &mut top_left_resources,
                    &mut bottom_right_resources,
                    r.top_left(),
                );
                make_bounding_box_include_point(
                    &mut top_left_resources,
                    &mut bottom_right_resources,
                    r.bottom_right(),
                );
            }

            let mut top_left_search_bounding_box =
                top_left_resources - dim_cc - max_tiles_between_command_center_and_resources;
            let mut bottom_right_search_bounding_box =
                bottom_right_resources + 1 + max_tiles_between_command_center_and_resources;

            make_point_fit_to_bounding_box(
                &mut top_left_search_bounding_box,
                self.top_left(),
                self.bottom_right() - dim_cc + 1,
            );
            make_point_fit_to_bounding_box(
                &mut bottom_right_search_bounding_box,
                self.top_left(),
                self.bottom_right() - dim_cc + 1,
            );

            // 2) Mark the Tiles with their distances from each remaining Ressource (Potential Fields >= 0)
            for &r in remaining_resources.iter() {
                let r = unsafe { &*r };
                for dy in -dim_cc.y - max_tiles_between_command_center_and_resources
                    ..r.size().y + dim_cc.y + max_tiles_between_command_center_and_resources
                {
                    for dx in -dim_cc.x - max_tiles_between_command_center_and_resources
                        ..r.size().x + dim_cc.x + max_tiles_between_command_center_and_resources
                    {
                        let t = r.top_left() + (dx, dy);
                        if map.valid(t) {
                            let tile = map.get_tile_mut(t);
                            let dist =
                                (dist_to_rectangle(t.center(), r.top_left(), r.size()) + 16) / 32;
                            let mut score =
                                0.max(max_tiles_between_command_center_and_resources + 3 - dist);
                            if r.is_geyser().is_some() {
                                // somewhat compensates for Geyser alone vs the several Minerals
                                score *= 3
                            }
                            if tile.area_id() == self.id() {
                                // note the additive effect (assume tile.InternalData() is 0 at the begining)
                                tile.set_internal_data(tile.internal_data() + score);
                            }
                        }
                    }
                }
            }

            // 3) Invalidate the 7 x 7 Tiles around each remaining Ressource (Starcraft rule)
            for &r in remaining_resources.iter() {
                let r = unsafe { &*r };
                for dx in -3..r.size().y + 3 {
                    for dy in -3..r.size().x + 3 {
                        let t = r.top_left() + (dx, dy);
                        if map.valid(t) {
                            map.get_tile_mut(t).set_internal_data(-1);
                        }
                    }
                }
            }

            // 4) Search the best location inside the SearchBoundingBox:
            let mut best_location = TilePosition::new(0, 0);
            let mut best_score = 0;
            let mut blocking_minerals = vec![];

            for y in top_left_search_bounding_box.y..bottom_right_search_bounding_box.y {
                for x in top_left_search_bounding_box.x..bottom_right_search_bounding_box.x {
                    let score = self.compute_base_location_score(TilePosition::new(x, y));
                    if score > best_score {
                        if self
                            .validate_base_location(TilePosition::new(x, y), &mut blocking_minerals)
                        {
                            best_score = score;
                            best_location = TilePosition::new(x, y);
                        }
                    }
                }
            }

            // 5) Clear Tile::m_internalData (required due to our use of Potential Fields: see comments in 2))
            for &r in remaining_resources.iter() {
                let r = unsafe { &*r };
                let r = unsafe { &*r };
                for dy in -dim_cc.y - max_tiles_between_command_center_and_resources
                    ..r.size().y + dim_cc.y + max_tiles_between_command_center_and_resources
                {
                    for dx in -dim_cc.x - max_tiles_between_command_center_and_resources
                        ..r.size().y + dim_cc.x + max_tiles_between_command_center_and_resources
                    {
                        let t = r.top_left() + TilePosition::new(dx, dy);
                        if map.valid(t) {
                            map.get_tile_mut(t).set_internal_data(0);
                        }
                    }
                }
            }

            if best_score == 0 {
                break;
            }

            // 6) Create a new Base at bestLocation, assign to it the relevant ressources and remove them from RemainingRessources:
            let mut assigned_resources = vec![];
            for (i, &r) in remaining_resources.iter().enumerate() {
                let res = unsafe { &*r };
                if dist_to_rectangle(res.pos(), best_location, dim_cc) + 2
                    <= max_tiles_between_command_center_and_resources * 32
                {
                    assigned_resources.push(r);
                }
            }
            remaining_resources.retain(|r| !assigned_resources.contains(&r));

            if assigned_resources.is_empty() {
                break;
            }

            self.bases.push(Base::new(
                self,
                best_location,
                assigned_resources,
                blocking_minerals,
            ));
        }
    }

    fn get_graph(&self) -> *mut Graph {
        self.p_graph
    }

    fn compute_base_location_score(&self, location: TilePosition) -> i32 {
        let map = unsafe { &mut *self.get_map() };
        let dim_cc = UnitType::Terran_Command_Center.tile_size();

        let mut sum_score = 0;
        for dy in 0..dim_cc.y {
            for dx in 0..dim_cc.x {
                let tile = map.get_tile(location + (dx, dy));
                if !tile.buildable() {
                    return -1;
                }
                if tile.internal_data() == -1 {
                    // The special value InternalData() == -1 means there is some ressource at maximum 3 tiles, which Starcraft rules forbid.
                    return -1;
                }
                if tile.area_id() != self.id() {
                    return -1;
                }
                if let Some(neutral) = unsafe { tile.get_neutral().as_ref() } {
                    if neutral.is_static_building().is_some() {
                        return -1;
                    }
                }
                sum_score += tile.internal_data();
            }
        }
        sum_score
    }

    // Checks if 'location' is a valid location for the placement of a Base Command Center.
    // If the location is valid except for the presence of Mineral patches of less than 9 (see Andromeda.scx),
    // the function returns true, and these Minerals are reported in BlockingMinerals
    // The function is intended to be called after ComputeBaseLocationScore, as it is more expensive.
    // See also the comments inside ComputeBaseLocationScore.
    fn validate_base_location(
        &self,
        location: TilePosition,
        blocking_minerals: &mut Vec<*const Mineral>,
    ) -> bool {
        let map = unsafe { &mut *self.get_map() };
        let dim_cc = UnitType::Terran_Command_Center.tile_size();

        blocking_minerals.clear();

        for dy in -3..dim_cc.y + 3 {
            for dx in -3..dim_cc.x + 3 {
                let t = location + (dx, dy);
                if map.valid(t) {
                    let tile = map.get_tile(t);
                    if let Some(n) = unsafe { tile.get_neutral().as_ref() } {
                        if n.is_geyser().is_some() {
                            return false;
                        }
                        if let Some(m) = n.is_mineral() {
                            let mineral = unsafe { &*m };
                            if mineral.initial_amount() <= 8 {
                                blocking_minerals.push(m);
                            } else {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        // checks the distance to the Bases already created:
        for base in self.bases() {
            if rounded_dist(base.location(), location) < min_tiles_between_bases {
                return false;
            }
        }
        true
    }

    fn compute_distances_starting_at(
        &self,
        start: TilePosition,
        targets: &[TilePosition],
    ) -> Vec<i32> {
        let map = unsafe { &mut *self.get_map() };
        let mut distances = vec![0; targets.len()];

        TileMark::unmark_all();

        #[derive(Eq, PartialEq)]
        struct DistPos(i32, TilePosition);
        impl Ord for DistPos {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                // Inverted to make it a min-heap
                other.0.cmp(&self.0)
            }
        }
        impl PartialOrd for DistPos {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut to_visit = BinaryHeap::new(); // a priority queue holding the tiles to visit ordered by their distance to start.
        to_visit.push(DistPos(0, start));
        let mut remaining_targets = targets.len();
        while let Some(DistPos(current_dist, current)) = to_visit.pop() {
            let current_tile = map.get_tile_mut(current);

            // Small change to original algorithm: We might have duplicated entries, see below
            if current_tile.mark.marked() {
                continue;
            }
            debug_assert!(current_tile.internal_data() == current_dist);
            current_tile.set_internal_data(0);
            // resets Tile::m_internalData for future usage
            current_tile.mark.set_marked();

            for i in 0..targets.len() {
                if current == targets[i] {
                    distances[i] = (0.5 + current_dist as f32 + 32.0 / 10000.0) as i32;
                    remaining_targets -= 1;
                }
            }
            if remaining_targets == 0 {
                break;
            }

            for delta in [
                TilePosition::new(-1, -1),
                TilePosition::new(0, -1),
                TilePosition::new(1, -1),
                TilePosition::new(-1, 0),
                TilePosition::new(1, 0),
                TilePosition::new(-1, 1),
                TilePosition::new(0, 1),
                TilePosition::new(1, 1),
            ] {
                let diagonal_move = delta.x != 0 && delta.y != 0;
                let new_next_dist = current_dist + if diagonal_move { 14142 } else { 10000 };

                let next = current + delta;
                if map.valid(next) {
                    let next_tile = map.get_tile_mut(next);
                    if !next_tile.mark.marked() {
                        if next_tile.internal_data() != 0 {
                            // next already in ToVisit
                            if new_next_dist < next_tile.internal_data() {
                                // nextNewDist < nextOldDist
                                // Change from original algorithm: We're not using a multimap, but a
                                // binary heap. Updating an entry is even slower here, so we just add
                                // it. See above on how it is skipped
                                next_tile.set_internal_data(new_next_dist);
                            }
                        } else if next_tile.area_id() == self.id() || next_tile.area_id() == -1 {
                            next_tile.set_internal_data(new_next_dist);
                            to_visit.push(DistPos(new_next_dist, next));
                        }
                    }
                }
            }
        }

        debug_assert_eq!(remaining_targets, 0);

        // Reset Tile::m_internalData for future usage
        for DistPos(_, tile_pos) in to_visit {
            map.get_tile_mut(tile_pos).set_internal_data(0);
        }
        distances
    }
}
