use super::{area::*, cp::*, defs::*, graph::*, neutral::*, tiles::*};
use crate::bwem::{defs::*, dist, norm, outer_mini_tile_border};
use crate::*;
use ahash::{AHashMap, AHashSet};
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::VecDeque;
use std::rc::Rc;

pub trait GetTile<T, P> {
    fn get_tile(&self, at: P) -> &T;
}

///////////////////////////////////////////////////////////////////////////////////////////////
///                                                                                          //
/// class Map
///                                                                                          //
///////////////////////////////////////////////////////////////////////////////////////////////
///
/// Map is the entry point:
///      - to access general information on the Map
///      - to access the Tiles and the MiniTiles
///      - to access the Areas
///      - to access the StartingLocations
///      - to access the Minerals, the Geysers and the StaticBuildings
///      - to parametrize the analysis process
///      - to update the information
/// Map also provides some useful tools such as Paths between ChokePoints and generic algorithms like BreadthFirstSearch
///
/// Map functionnality is provided through its singleton Map::Instance().
pub struct Map {
    size: i32,
    Size: TilePosition,
    walk_size: i32,
    Walk_size: WalkPosition,
    center: Position,
    tiles: Vec<Tile>,
    mini_tiles: Vec<MiniTile>,
    raw_frontier: RefCell<Vec<((AreaId, AreaId), WalkPosition)>>,
    automatic_path_update: bool,
    starting_locations: Vec<TilePosition>,
    minerals: Vec<Box<Mineral>>,
    geysers: Vec<Box<Geyser>>,
    static_buildings: Vec<Box<StaticBuilding>>,
    max_altitude: Altitude,
}

impl GetTile<Tile, TilePosition> for Map {
    fn get_tile(&self, pos: TilePosition) -> &Tile {
        self.get_tile(pos)
    }
}

impl Map {
    /// This has to be called before any other function is called.
    /// A good place to do this is in ExampleAIModule::onStart()
    pub fn initialize(&mut self, game: &Game) {
        self.Size = TilePosition::new(game.map_width(), game.map_height());
        self.size = self.Size.x * self.Size.y;
        self.tiles.resize_with(self.size as usize, Default::default);

        self.Walk_size = self.Size.to_walk_position();
        self.walk_size = self.Walk_size.x * self.Walk_size.y;
        self.mini_tiles
            .resize_with(self.walk_size as usize, Default::default);
        self.center = self.Size.to_position() / 2;

        for t in game.get_start_locations() {
            self.starting_locations.push(t);
        }

        self.load_data(game);

        self.decide_sea_or_lakes();

        self.initialize_neutrals(game);

        self.compute_altitude();

        self.process_blocking_neutrals();

        self.compute_areas();

        let graph = unsafe { &mut *self.get_graph() };
        graph.create_choke_points();

        graph.compute_choke_point_distance_matrix();

        graph.collect_information();

        graph.create_bases();
    }

    // Computes walkability, buildability and groundHeight and doodad information, using BWAPI corresponding functions
    fn load_data(&mut self, game: &Game) {
        // Mark unwalkable minitiles (minitiles are walkable by default)
        for y in 0..self.Walk_size.y {
            for x in 0..self.Walk_size.x {
                if !game.is_walkable((x, y)) {
                    // For each unwalkable minitile, we also mark its 8 neighbours as not walkable.
                    for dy in -1..=1 {
                        // According to some tests, this prevents from wrongly pretending one Marine can go by some thin path.
                        for dx in -1..=1 {
                            let w = WalkPosition::new(dx + x, dy + y);
                            if self.valid(w) {
                                self.get_mini_tile(w).set_walkable(false);
                            }
                        }
                    }
                }
            }
        }

        // Mark buildable tiles (tiles are unbuildable by default)
        for y in 0..self.Size.y {
            for x in 0..self.Size.x {
                let t = TilePosition::new(x, y);

                // Set buildable
                let buildable = game.is_buildable(t);
                if buildable {
                    self.get_tile_mut(t).set_buildable();
                }

                // Check if tile is fully walkable
                let mut walkable = true;
                'walkable: for dy in 0..4 {
                    for dx in 0..4 {
                        let w = t.to_walk_position() + (dx, dy);
                        if !game.is_walkable(w) {
                            walkable = false;
                            break 'walkable;
                        }
                    }
                }

                // Set walkable if buildable or fully walkable
                if buildable || walkable {
                    for dy in 0..4 {
                        for dx in 0..4 {
                            self.get_mini_tile(t.to_walk_position() + (dx, dy))
                                .set_walkable(true);
                        }
                    }
                }

                // Add groundHeight and doodad information
                let bwapi_ground_height = game.get_ground_height(t);
                self.get_tile_mut(t)
                    .set_ground_height(bwapi_ground_height / 2);
                if bwapi_ground_height % 2 != 0 {
                    self.get_tile_mut(t).set_doodad();
                }
            }
        }
    }

    fn decide_sea_or_lakes(&mut self) {
        let mut to_search: Vec<WalkPosition> = vec![];
        let mut sea_extent: Vec<WalkPosition> = vec![];
        for y in 0..self.Walk_size.y {
            for x in 0..self.Walk_size.x {
                let origin = WalkPosition::new(x, y);
                let Origin = self.get_mini_tile(origin);
                if Origin.sea_or_lake() {
                    to_search.clear();
                    sea_extent.clear();
                    Origin.set_sea();
                    let mut top_left = origin;
                    let mut bottom_right = origin;
                    while let Some(current) = to_search.pop() {
                        top_left.x = top_left.x.min(current.x);
                        top_left.y = top_left.y.min(current.y);
                        bottom_right.x = bottom_right.x.max(current.x);
                        bottom_right.y = bottom_right.y.max(current.y);

                        for delta in [
                            WalkPosition::new(0, -1),
                            WalkPosition::new(-1, 0),
                            WalkPosition::new(1, 0),
                            WalkPosition::new(0, 1),
                        ] {
                            let next = current + delta;
                            if self.valid(next) {
                                let Next = self.get_mini_tile(next);
                                if Next.sea_or_lake() {
                                    to_search.push(next);
                                    Next.set_sea();
                                    if sea_extent.len() <= lake_max_mini_tiles as usize {
                                        sea_extent.push(next);
                                    }
                                }
                            }
                        }
                    }

                    if sea_extent.len() <= lake_max_mini_tiles as usize
                        && bottom_right.x - top_left.x <= lake_max_width_in_mini_tiles
                        && bottom_right.y - top_left.y <= lake_max_width_in_mini_tiles
                        && top_left.x >= 2
                        && top_left.y >= 2
                        && bottom_right.x < self.Walk_size.x - 2
                        && bottom_right.y < self.Walk_size.y - 2
                    {
                        for &sea_mini_tile in sea_extent.iter() {
                            self.get_mini_tile(sea_mini_tile).set_lake();
                        }
                    }
                }
            }
        }
    }

    fn initialize_neutrals(&mut self, game: &Game) {
        for n in game.get_static_neutral_units() {
            if n.get_type().is_building() {
                if n.get_type().is_mineral_field() {
                    self.minerals.push(Box::new(Mineral::new(&n, self)));
                } else if n.get_type() == UnitType::Resource_Vespene_Geyser {
                    self.geysers.push(Box::new(Geyser::new(&n, self)));
                } else {
                    // Let's ignore buildings which are not special buildings.
                    // They should be destroyed as part of regular battle.
                    debug_assert!(
                        n.get_type().is_special_building(),
                        "Building {} at position {} is not special",
                        n.get_type().name(),
                        n.get_position()
                    );
                    self.static_buildings
                        .push(Box::new(StaticBuilding::new(&n, self)));
                }
            } else if n.get_type() != UnitType::Zerg_Egg {
                if !n.get_type().is_critter() {
                    debug_assert!(
                        !n.get_type().is_special_building(),
                        "{}",
                        n.get_type().name()
                    );

                    debug_assert!(
                        n.get_type() == UnitType::Special_Pit_Door
                            || n.get_type() == UnitType::Special_Right_Pit_Door,
                        "Unit {} at position {} is not XXX_Pit_Door",
                        n.get_type().name(),
                        n.get_position()
                    );
                    if n.get_type() == UnitType::Special_Pit_Door {
                        self.static_buildings
                            .push(Box::new(StaticBuilding::new(&n, self)));
                    }
                    if n.get_type() == UnitType::Special_Right_Pit_Door {
                        self.static_buildings
                            .push(Box::new(StaticBuilding::new(&n, self)));
                    }
                }
            }
        }
    }

    // Assigns MiniTile::m_altitude foar each miniTile having AltitudeMissing()
    // Cf. MiniTile::Altitude() for meaning of altitude_t.
    // Altitudes are computed using the straightforward Dijkstra's algorithm : the lower ones are computed first, starting from the seaside-miniTiles neighbours.
    // The point here is to precompute all possible altitudes for all possible tiles, and sort them.
    fn compute_altitude(&mut self) {
        let altitude_scale = 8; // 8 provides a pixel definition for altitude_t, since altitudes are computed from miniTiles which are 8x8 pixels

        // 1) Fill in and sort DeltasByAscendingAltitude
        let range = self.Walk_size.x.max(self.Walk_size.y) / 2 + 3;

        let mut deltas_by_ascending_altitude = vec![];

        for dy in 0..=range {
            for dx in dy..=range {
                // Only consider 1/8 of possible deltas. Other ones obtained by symmetry.
                if dx != 0 || dy != 0 {
                    deltas_by_ascending_altitude.push((
                        WalkPosition::new(dx, dy),
                        (0.5 + norm(dx, dy) * altitude_scale as f64) as Altitude,
                    ));
                }
            }
        }
        deltas_by_ascending_altitude.sort_by_key(|(_, a)| *a);

        // 2) Fill in ActiveSeaSideList, which basically contains all the seaside miniTiles (from which altitudes are to be computed)
        //    It also includes extra border-miniTiles which are considered as seaside miniTiles too.
        struct ActiveSeaSide {
            origin: WalkPosition,
            last_altitude_generated: Altitude,
        }
        let mut active_sea_side_list = vec![];

        for y in -1..=self.Walk_size.y {
            for x in -1..=self.Walk_size.x {
                let w = WalkPosition::new(x, y);
                if !self.valid(w) || self.sea_side(w) {
                    active_sea_side_list.push(ActiveSeaSide {
                        origin: w,
                        last_altitude_generated: 0,
                    });
                }
            }
        }

        // 3) Dijkstra's algorithm
        for (d, altitude) in deltas_by_ascending_altitude {
            let mut i = 0;
            while i < active_sea_side_list.len() {
                let Current = &mut active_sea_side_list[i];
                if altitude - Current.last_altitude_generated >= 2 * altitude_scale {
                    // optimization : once a seaside miniTile verifies this condition,
                    active_sea_side_list.swap_remove(i); // we can throw it away as it will not generate min altitudes anymore
                } else {
                    for delta in [
                        WalkPosition::new(d.x, d.y),
                        WalkPosition::new(-d.x, d.y),
                        WalkPosition::new(d.x, -d.y),
                        WalkPosition::new(-d.x, -d.y),
                    ] {
                        let w = Current.origin + delta;
                        if self.valid(w) {
                            let mini_tile = self.get_mini_tile(w);
                            if mini_tile.altitude_missing() {
                                Current.last_altitude_generated = altitude;
                                mini_tile.set_altitude(altitude);
                                self.max_altitude = altitude;
                            }
                        }
                    }
                    i += 1;
                }
            }
        }
    }

    fn sea_side(&self, p: WalkPosition) -> bool {
        if self.get_mini_tile(p).sea() {
            return false;
        }

        for delta in [
            WalkPosition::new(0, -1),
            WalkPosition::new(-1, 0),
            WalkPosition::new(1, 0),
            WalkPosition::new(0, 1),
        ] {
            if self.valid(p + delta) {
                if self.get_mini_tile(p + delta).sea() {
                    return true;
                }
            }
        }
        return false;
    }

    fn process_blocking_neutrals(&mut self) {
        let mut candidates: Vec<&dyn Neutral> = vec![];
        for s in self.static_buildings() {
            candidates.push(s.as_ref());
        }
        for m in self.minerals() {
            candidates.push(m.as_ref());
        }

        let mut to_visit = vec![];
        let mut visited = AHashSet::new();
        let mut true_doors = vec![];

        for candidate in candidates {
            if candidate.next_stacked().is_null() {
                // in the case where several neutrals are stacked, we only consider the top one
                // 1)  Retreave the Border: the outer border of pCandidate
                let mut border = outer_mini_tile_border(
                    candidate.top_left().to_walk_position(),
                    candidate.size().to_walk_position(),
                );

                // 2)  Find the doors in Border: one door for each connected set of walkable, neighbouring miniTiles.
                //     The searched connected miniTiles all have to be next to some lake or some static building, though they can't be part of one.
                let mut Doors = vec![];
                while let Some(door) = border.pop() {
                    Doors.push(door);

                    to_visit.clear();
                    to_visit.push(door);
                    visited.clear();
                    visited.insert(door);
                    while let Some(current) = to_visit.pop() {
                        for delta in [
                            WalkPosition::new(0, -1),
                            WalkPosition::new(-1, 0),
                            WalkPosition::new(1, 0),
                            WalkPosition::new(0, 1),
                        ] {
                            let next = current + delta;
                            if self.valid(next) && !visited.contains(&next) {
                                if self.get_mini_tile(next).walkable() {
                                    if self
                                        .get_tile(next.to_tile_position())
                                        .get_neutral()
                                        .is_null()
                                    {
                                        if self.adjoins_8_some_lake_or_neutral(next) {
                                            to_visit.push(next);
                                            visited.insert(next);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    border.retain(|w| !visited.contains(w));
                }

                // 3)  If at least 2 doors, find the true doors in Border: a true door is a door that gives onto an area big enough
                true_doors.clear();
                if Doors.len() >= 2 {
                    for door in Doors {
                        to_visit.clear();
                        to_visit.push(door);
                        visited.clear();
                        visited.insert(door);
                        let limit = if candidate.is_static_building().is_some() {
                            10
                        } else {
                            400
                        };
                        while !to_visit.is_empty() && visited.len() < limit {
                            let current = to_visit.pop().unwrap();
                            for delta in [
                                WalkPosition::new(0, -1),
                                WalkPosition::new(-1, 0),
                                WalkPosition::new(1, 0),
                                WalkPosition::new(0, 1),
                            ] {
                                let next = current + delta;
                                if self.valid(next) && !visited.contains(&next) {
                                    if self.get_mini_tile(next).walkable() {
                                        if self
                                            .get_tile(next.to_tile_position())
                                            .get_neutral()
                                            .is_null()
                                        {
                                            to_visit.push(next);
                                            visited.insert(next);
                                        }
                                    }
                                }
                            }
                        }
                        if visited.len() >= limit {
                            true_doors.push(door);
                        }
                    }

                    // 4)  If at least 2 true doors, pCandidate is a blocking static building
                    if true_doors.len() >= 2 {
                        // Marks pCandidate (and any Neutral stacked with it) as blocking.
                        let mut p_neutral = self.get_tile(candidate.top_left()).get_neutral();
                        while !p_neutral.is_null() {
                            let neutral = unsafe { &mut *p_neutral };
                            neutral.set_blocking(&true_doors);
                            p_neutral = neutral.next_stacked();
                        }

                        // Marks all the miniTiles of pCandidate as blocked.
                        // This way, areas at TrueDoors won't merge together.
                        for dy in 0..candidate.size().to_walk_position().y {
                            for dx in 0..candidate.size().to_walk_position().x {
                                let mini_tile = self.get_mini_tile(
                                    candidate.top_left().to_walk_position() + (dx, dy),
                                );
                                if mini_tile.walkable() {
                                    mini_tile.set_blocked();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn adjoins_8_some_lake_or_neutral(&self, p: WalkPosition) -> bool {
        for delta in [
            WalkPosition::new(-1, 1),
            WalkPosition::new(0, -1),
            WalkPosition::new(1, -1),
            WalkPosition::new(-1, 0),
            WalkPosition::new(-1, 1),
            WalkPosition::new(0, 1),
            WalkPosition::new(1, 1),
        ] {
            let next = p + delta;
            if self.valid(next) {
                if !self
                    .get_tile(next.to_tile_position())
                    .get_neutral()
                    .is_null()
                {
                    return true;
                }
                if self.get_mini_tile(next).lake() {
                    return true;
                }
            }
        }
        return false;
    }

    fn compute_areas(&mut self) {
        let mini_tiles_by_descending_altitude = self.sort_mini_tiles();
        let temp_area_list = self.compute_temp_areas(mini_tiles_by_descending_altitude);
        self.create_areas(temp_area_list);
        unimplemented!()
    }

    // Initializes m_Graph with the valid and big enough areas in TempAreaList.
    fn create_areas(&self, temp_area_list: Vec<TempAreaInfo>) {
        let mut areas_list = vec![];

        let mut new_area_id = 1;
        let mut new_tiny_area_id = -2;

        for temp_area in temp_area_list {
            if temp_area.valid() {
                if temp_area.size >= area_min_mini_tiles {
                    debug_assert!(new_area_id <= temp_area.id());
                    if new_area_id != temp_area.id() {
                        self.replace_area_ids(temp_area.top(), new_area_id);
                    }
                    areas_list.push((temp_area.top(), temp_area.size));
                    new_area_id += 1;
                } else {
                    self.replace_area_ids(temp_area.top(), new_tiny_area_id);
                    new_tiny_area_id += 1;
                }
            }
        }
        unsafe { &mut *self.get_graph() }.create_areas(areas_list);
    }

    fn get_graph(&self) -> *mut Graph {
        unimplemented!()
    }

    fn sort_mini_tiles(&self) -> Vec<(WalkPosition, &MiniTile)> {
        let mut mini_tiles_by_descending_altitude = vec![];
        for y in 0..self.Walk_size.y {
            for x in 0..self.Walk_size.x {
                let w = WalkPosition::new(x, y);
                let mini_tile = self.get_mini_tile(w);
                if mini_tile.area_id_missing() {
                    mini_tiles_by_descending_altitude.push((w, mini_tile));
                }
            }
        }
        mini_tiles_by_descending_altitude.sort_by_key(|(_, mt)| Reverse(mt.altitude()));
        mini_tiles_by_descending_altitude
    }

    fn compute_temp_areas(
        &self,
        mini_tiles_by_descending_altitude: Vec<(WalkPosition, &MiniTile)>,
    ) -> Vec<TempAreaInfo> {
        let mut map_area_pair_counter = AHashMap::new();
        fn choose_neighboring_area(
            map_area_pair_counter: &mut AHashMap<(usize, usize), i32>,
            mut a: usize,
            mut b: usize,
        ) -> usize {
            if a > b {
                std::mem::swap(&mut a, &mut b);
            }
            let counter = map_area_pair_counter.entry((a, b)).or_insert(0);
            let result = if *counter % 2 == 0 { a } else { b };
            *counter += 1;
            result
        }

        let mut temp_area_list = vec![Default::default()]; // TempAreaList[0] left unused, as AreaIds are > 0
        for (pos, cur) in mini_tiles_by_descending_altitude {
            let neighboring_areas = self.find_neighboring_areas(pos);
            if neighboring_areas.0 == 0 {
                // no neighboring area : creates of a new area
                temp_area_list.push(TempAreaInfo::new(temp_area_list.len() as AreaId, cur, pos));
            } else if neighboring_areas.1 == 0 {
                // one neighboring area : adds cur to the existing area
                temp_area_list[neighboring_areas.0 as usize].add(cur);
            } else {
                // two neighboring areas : adds cur to one of them  &  possible merging
                let mut smaller = neighboring_areas.0 as usize;
                let mut bigger = neighboring_areas.1 as usize;

                if temp_area_list[smaller].size > temp_area_list[bigger].size {
                    std::mem::swap(&mut smaller, &mut bigger);
                }
                // Condition for the neighboring areas to merge:
                if temp_area_list[smaller].size < 80
                    || temp_area_list[smaller].highest_altitude() < 80
                    || cur.altitude() as f64 / temp_area_list[bigger].highest_altitude as f64
                        >= 0.90
                    || cur.altitude() as f64 / temp_area_list[smaller].highest_altitude as f64
                        >= 0.90
                    || self.starting_locations.iter().any(|&starting_loc| {
                        dist(pos.to_tile_position(), starting_loc + (2, 1)) <= 3.0
                    })
                {
                    // adds cur to the absorbing area:
                    temp_area_list[bigger].add(cur);

                    // merges the two neighboring areas:
                    self.replace_area_ids(temp_area_list[smaller].top(), bigger as AreaId);
                    assert!(bigger != smaller);
                    let (a, b) = (
                        unsafe {
                            &mut *(&temp_area_list[bigger] as *const TempAreaInfo
                                as *mut TempAreaInfo)
                        },
                        unsafe {
                            &mut *(&temp_area_list[smaller] as *const TempAreaInfo
                                as *mut TempAreaInfo)
                        },
                    );
                    a.merge(b);
                } else {
                    // no merge : cur starts or continues the frontier between the two neighboring areas
                    // adds cur to the chosen Area:

                    temp_area_list
                        [choose_neighboring_area(&mut map_area_pair_counter, smaller, bigger)]
                    .add(cur);
                    self.raw_frontier
                        .borrow_mut()
                        .push((neighboring_areas, pos));
                }
            }
        }

        // Remove from the frontier obsolete positions
        self.raw_frontier.borrow_mut().retain(|f| f.0 .0 != f.0 .1);
        temp_area_list
    }

    fn replace_area_ids(&self, p: WalkPosition, new_area_id: AreaId) {
        let origin = self.get_mini_tile(p);
        let old_area_id = origin.area_id();
        origin.replace_area_id(new_area_id);

        let mut to_search = vec![p];
        while let Some(current) = to_search.pop() {
            for delta in [
                WalkPosition::new(0, -1),
                WalkPosition::new(-1, 0),
                WalkPosition::new(1, 0),
                WalkPosition::new(0, 1),
            ] {
                let next = current + delta;
                if self.valid(next) {
                    let Next = self.get_mini_tile(next);
                    if Next.area_id() == old_area_id {
                        to_search.push(next);
                        Next.replace_area_id(new_area_id);
                    }
                }
            }
        }

        // also replaces references of oldAreaId by newAreaId in m_RawFrontier:
        if new_area_id > 0 {
            for f in self.raw_frontier.borrow_mut().iter_mut() {
                if f.0 .0 == old_area_id {
                    f.0 .0 = new_area_id
                }
                if f.0 .1 == old_area_id {
                    f.0 .1 = new_area_id
                }
            }
        }
    }

    fn find_neighboring_areas(&self, p: WalkPosition) -> (AreaId, AreaId) {
        let mut result = (0, 0);
        for delta in [
            WalkPosition::new(0, -1),
            WalkPosition::new(-1, 0),
            WalkPosition::new(1, 0),
            WalkPosition::new(0, 1),
        ] {
            if self.valid(p + delta) {
                let area_id = self.get_mini_tile(p + delta).area_id();
                if area_id > 0 {
                    if result.0 != 0 {
                        result.0 = area_id
                    } else if result.0 != area_id {
                        if result.1 == 0 || area_id < result.1 {
                            result.1 = area_id;
                        }
                    }
                }
            }
        }
        result
    }

    // Will return true once Initialize() has been called.
    pub fn initialized(&self) -> bool {
        self.size != 0
    }

    /// Returns the status of the automatic path update (off (false) by default).
    /// When on, each time a blocking Neutral (either Mineral or StaticBuilding) is destroyed,
    /// any information relative to the paths through the Areas is updated accordingly.
    /// For this to function, the Map still needs to be informed of such destructions
    /// (by calling OnMineralDestroyed and OnStaticBuildingDestroyed).
    pub fn automatic_path_update(&self) -> bool {
        self.automatic_path_update
    }

    /// Enables the automatic path update (Cf. AutomaticPathUpdate()).
    /// One might NOT want to call this function, in order to make the accessibility between Areas remain the same throughout the game.
    /// Even in this case, one should keep calling OnMineralDestroyed and OnStaticBuildingDestroyed.
    pub fn enable_automatic_path_analysis(&mut self) {
        self.automatic_path_update = true;
    }

    /// Tries to assign one Base for each starting Location in StartingLocations().
    /// Only nearby Bases can be assigned (Cf. detail::max_tiles_between_StartingLocation_and_its_AssignedBase).
    /// Each such assigned Base then has Starting() == true, and its Location() is updated.
    /// Returns whether the function succeeded (a fail may indicate a failure in BWEM's Base placement analysis
    /// or a suboptimal placement in one of the starting Locations).
    /// You normally should call this function, unless you want to compare the StartingLocations() with
    /// BWEM's suggested locations for the Bases.
    pub fn find_bases_for_starting_locations(&mut self) -> bool {
        unimplemented!()
    }

    /// Returns the size of the Map in Tiles.
    pub fn size(&self) -> TilePosition {
        self.Size
    }

    /// Returns the size of the Map in MiniTiles.
    pub fn walk_size(&self) -> WalkPosition {
        self.Walk_size
    }

    /// Returns the center of the Map in pixels.
    pub fn center(&self) -> Position {
        self.center
    }

    /// Returns a random position in the Map in pixels.
    pub fn random_position(&self) -> Position {
        unimplemented!()
    }

    /// Returns the maximum altitude in the whole Map (Cf. MiniTile::Altitude()).
    pub fn max_altitude(&self) -> Altitude {
        unimplemented!()
    }

    /// Returns the number of Bases.
    pub fn base_count(&self) -> isize {
        unimplemented!()
    }

    /// Returns the number of ChokePoints.
    pub fn choke_point_count(&self) -> isize {
        unimplemented!()
    }

    /// Returns a Tile, given its position.
    pub fn get_tile(&self, p: TilePosition) -> &Tile {
        debug_assert!(self.valid(p));
        &self.tiles[(self.size().x * p.y + p.x) as usize]
    }

    /// Returns a Tile, given its position.
    pub fn get_tile_mut(&mut self, p: TilePosition) -> &mut Tile {
        debug_assert!(self.valid(p));
        let size = self.size();
        &mut self.tiles[(size.x * p.y + p.x) as usize]
    }

    /// Returns a MiniTile, given its position.
    pub fn get_mini_tile(&self, p: WalkPosition) -> &MiniTile {
        debug_assert!(self.valid(p));
        &self.mini_tiles[(self.walk_size().x * p.x + p.y) as usize]
    }

    /// Provides access to the internal array of Tiles.
    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    /// Provides access to the internal array of MiniTiles.
    pub fn mini_tiles(&self) -> &[MiniTile] {
        &self.mini_tiles
    }

    /// Returns whether the position p is valid.
    pub fn valid<const N: i32>(&self, p: ScaledPosition<N>) -> bool {
        match N {
            32 => 0 <= p.x && p.x < self.size().x && 0 <= p.y && p.y < self.size().y,
            8 => 0 <= p.x && p.x < self.walk_size().x && 0 <= p.y && p.y < self.walk_size().y,
            1 => self.valid(WalkPosition::new(p.x / 8, p.y / 8)),
            _ => false,
        }
    }

    /// Returns the position closest to p that is valid.
    pub fn crop_wp(&self, p: WalkPosition) -> WalkPosition {
        unimplemented!()
    }
    pub fn crop_tp(&self, p: TilePosition) -> TilePosition {
        unimplemented!()
    }
    pub fn crop(&self, p: Position) -> Position {
        unimplemented!()
    }

    /// Returns a reference to the starting Locations.
    /// Note: these correspond to BWAPI::getStartLocations().
    pub fn starting_locations(&self) -> &[TilePosition] {
        unimplemented!()
    }

    /// Returns a reference to the Minerals (Cf. Mineral).
    pub fn minerals(&self) -> &[Box<Mineral>] {
        &self.minerals
    }

    /// Returns a reference to the Geysers (Cf. Geyser).
    pub fn geysers(&self) -> &[Rc<RefCell<Geyser>>] {
        unimplemented!()
    }

    /// Returns a reference to the StaticBuildings (Cf. StaticBuilding).
    pub fn static_buildings(&self) -> &[Box<StaticBuilding>] {
        &self.static_buildings
    }
    ///
    ///
    /// If a Mineral wrappers the given BWAPI unit, returns a pointer to it.
    /// Otherwise, returns nullptr.
    pub fn get_mineral(&self, unit: Unit) -> Option<Rc<RefCell<Mineral>>> {
        unimplemented!()
    }

    /// If a Geyser wrappers the given BWAPI unit, returns a pointer to it.
    /// Otherwise, returns nullptr.
    pub fn get_geyser(&self, unit: Unit) -> Option<Rc<RefCell<Geyser>>> {
        unimplemented!()
    }

    /// Should be called for each destroyed BWAPI unit u having u->getType().isMineralField() == true
    pub fn on_mineral_destroyed(&mut self, unit: Unit) {
        unimplemented!()
    }

    /// Should be called for each destroyed BWAPI unit u having u->getType().isSpecialBuilding() == true
    pub fn on_static_building_destroyed(&mut self, unit: Unit) {
        unimplemented!()
    }

    /// Returns a reference to the Areas.
    pub fn areas(&self) -> &[Area] {
        unimplemented!()
    }
    /// Returns an Area given its id.
    pub fn get_area_by_id(&self, id: AreaId) -> Rc<RefCell<Area>> {
        unimplemented!()
    }

    /// If the MiniTile at w is walkable and is part of an Area, returns that Area.
    /// Otherwise, returns nullptr;
    /// Note: because of the lakes, GetNearestArea should be prefered over GetArea.
    pub fn get_area_by_wp(&self, w: WalkPosition) -> Rc<RefCell<Area>> {
        unimplemented!()
    }

    /// If the Tile at t contains walkable sub-MiniTiles which are all part of the same Area, returns that Area.
    /// Otherwise, returns nullptr;
    /// Note: because of the lakes, GetNearestArea should be prefered over GetArea.
    pub fn get_area_by_tp(&self, t: TilePosition) -> Rc<RefCell<Area>> {
        unimplemented!()
    }

    /// Returns the nearest Area from w.
    /// Returns nullptr only if Areas().empty()
    /// Note: Uses a breadth first search.
    pub fn get_nearest_area<const N: i32>(
        &self,
        p: ScaledPosition<N>,
    ) -> Option<Rc<RefCell<Area>>> {
        match N {
            8 => unimplemented!(),
            32 => unimplemented!(),
            _ => panic!(),
        }
    }

    /// Returns a list of ChokePoints, which is intended to be the shortest walking path from 'a' to 'b'.
    /// Furthermore, if pLength != nullptr, the pointed integer is set to the corresponding length in pixels.
    /// If 'a' is not accessible from 'b', the empty Path is returned, and -1 is put in *pLength (if pLength != nullptr).
    /// If 'a' and 'b' are in the same Area, the empty Path is returned, and a.getApproxDistance(b) is put in *pLength (if pLength != nullptr).
    /// Otherwise, the function relies on ChokePoint::GetPathTo.
    /// Cf. ChokePoint::GetPathTo for more information.
    /// Note: in order to retrieve the Areas of 'a' and 'b', the function starts by calling
    ///       GetNearestArea(TilePosition(a)) and GetNearestArea(TilePosition(b)).
    ///       While this brings robustness, this could yield surprising results in the case where 'a' and/or 'b' are in the Water.
    ///       To avoid this and the potential performance penalty, just make sure GetArea(a) != nullptr and GetArea(b) != nullptr.
    ///       Then GetPath should perform very quick.
    pub fn get_path(&self, a: Position, b: Position, length: Option<&mut isize>) -> &Path {
        unimplemented!()
    }

    /// Generic algorithm for breadth first search in the Map.
    /// See the several use cases in BWEM source files.
    pub fn breadth_first_search<
        T,
        P1: Fn(&T, ScaledPosition<N>) -> bool,
        P2: Fn(&T, ScaledPosition<N>) -> bool,
        const N: i32,
    >(
        &self,
        start: ScaledPosition<N>,
        find_cond: P1,
        visit_cond: P2,
        connect8: bool,
    ) -> ScaledPosition<N>
    where
        Self: GetTile<T, ScaledPosition<N>>,
    {
        if find_cond(GetTile::get_tile(self, start), start) {
            return start;
        }

        let mut visited = AHashSet::new();
        let mut to_visit = VecDeque::new();

        to_visit.push_back(start);
        visited.insert(start);

        let dir8 = [
            ScaledPosition::<N>::new(-1, -1),
            ScaledPosition::<N>::new(0, -1),
            ScaledPosition::<N>::new(1, -1),
            ScaledPosition::<N>::new(-1, 0),
            ScaledPosition::<N>::new(1, 0),
            ScaledPosition::<N>::new(-1, 1),
            ScaledPosition::<N>::new(0, 1),
            ScaledPosition::<N>::new(1, 1),
        ];
        let dir4 = [
            ScaledPosition::<N>::new(0, -1),
            ScaledPosition::<N>::new(-1, 0),
            ScaledPosition::<N>::new(1, 0),
            ScaledPosition::<N>::new(0, 1),
        ];
        let directions: &[_] = if connect8 { &dir8 } else { &dir4 };

        while let Some(current) = to_visit.pop_front() {
            for &delta in directions {
                let next = current + delta;
                if self.valid(next) {
                    let next_tile = GetTile::get_tile(self, next);
                    if find_cond(next_tile, next) {
                        return next;
                    }

                    if visit_cond(next_tile, next) && visited.contains(&next) {
                        to_visit.push_back(next);
                        visited.insert(next);
                    }
                }
            }
        }

        panic!("Search failed")
    }
    // template<class TPosition, class Pred1, class Pred2>
    // TPosition                                                       BreadthFirstSearch(TPosition start, Pred1 findCond, Pred2 visitCond, bool connect8 = true) const;

    /// Returns the union of the geometry of all the ChokePoints. Cf. ChokePoint::Geometry()
    pub fn raw_frontier(&self) -> &[((AreaId, AreaId), WalkPosition)] {
        unimplemented!()
    }

    pub fn draw(&self, game: &Game) {
        unimplemented!()
    }
}

// Helper class for void Map::ComputeAreas()
// Maintains some information about an area being computed
// A TempAreaInfo is not Valid() in two cases:
//   - a default-constructed TempAreaInfo instance is never Valid (used as a dummy value to simplify the algorithm).
//   - any other instance becomes invalid when absorbed (see Merge)
#[derive(Default)]
struct TempAreaInfo {
    valid: bool,
    id: AreaId,
    top: WalkPosition,
    highest_altitude: Altitude,
    size: i32,
}

impl TempAreaInfo {
    fn new(id: AreaId, mini_tile: &MiniTile, pos: WalkPosition) -> Self {
        Self {
            valid: true,
            id: id,
            top: pos,
            size: 0,
            highest_altitude: mini_tile.altitude(),
        }
    }

    fn valid(&self) -> bool {
        self.valid
    }

    fn id(&self) -> AreaId {
        debug_assert!(self.valid);
        self.id
    }

    fn top(&self) -> WalkPosition {
        debug_assert!(self.valid);
        self.top
    }

    fn highest_altitude(&self) -> Altitude {
        debug_assert!(self.valid);
        self.highest_altitude
    }

    fn add(&mut self, mini_tile: &MiniTile) {
        debug_assert!(self.valid);
        self.size += 1;
        mini_tile.set_area_id(self.id);
    }

    fn merge(&mut self, absorbed: &mut TempAreaInfo) {
        debug_assert!(self.valid && absorbed.valid);
        debug_assert!(self.size >= absorbed.size);
        self.size += absorbed.size;
        absorbed.valid = false;
    }
}
