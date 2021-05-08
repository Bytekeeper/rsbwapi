use super::*;
use bitfield::bitfield;
use std::collections::{HashMap, VecDeque};

type Altitude = i16;
type AreaId = i16;
type GroupId = i16;
type ChokePointId = i16;

const LAKE_MAX_WIDTH_IN_MINI_TILES: i32 = 8 * 4;
const LAKE_MAX_MINI_TILES: usize = 300;
const AREA_MIN_MINI_TILES: i32 = 64;
const BLOCKING_CP: i16 = i16::MAX;

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
#[derive(Clone)]
pub struct ChokePoint {
    geometry: Vec<WalkPosition>,
    middle: WalkPosition,
}

impl ChokePoint {
    fn new(map: &Map, index: u32, a: AreaId, b: AreaId, geometry: &[WalkPosition]) -> Self {
        debug_assert!(geometry.is_empty());
        Self {
            geometry: geometry.to_vec(),
            middle: *geometry
                .iter()
                .max_by_key(|&&p| map.mini_tiles[p].altitude)
                .unwrap(),
        }
    }
}
/// Areas are regions that BWEM automatically computes from Brood War's maps
/// Areas aim at capturing relevant regions that can be walked, though they may contain small inner non walkable regions called lakes.
/// More formally:
///  - An area consists in a set of 4-connected MiniTiles, which are either Terrain-MiniTiles or Lake-MiniTiles.
///  - An Area is delimited by the side of the Map, by Water-MiniTiles, or by other Areas. In the latter case
///    the adjoining Areas are called neighbouring Areas, and each pair of such Areas defines at least one ChokePoint.
/// Like ChokePoints and Bases, the number and the addresses of Area instances remain unchanged.
/// To access Areas one can use their ids or their addresses with equivalent efficiency.
///
/// Areas inherit utils::Markable, which provides marking ability
/// Areas inherit utils::UserData, which provides free-to-use data.
#[derive(Default)]
pub struct Area {
    id: AreaId,
    group_id: GroupId,
    top: WalkPosition,
    area: Rectangle<TilePosition>,
    max_altitude: Altitude,
    mini_tiles: i32,
    tiles: i32,
    buildable_tiles: i32,
    high_ground_tiles: i32,
    very_high_ground_tiles: i32,
    choke_points_by_area: HashMap<AreaId, Vec<ChokePointId>>,
    accessible_neighbours: Vec<AreaId>,
    choke_points: Vec<ChokePointId>,
    minerals: Vec<usize>,
    geysers: Vec<usize>,
    bases: Vec<Base>,
}

impl Area {
    fn new(map: &Map, id: AreaId, top: WalkPosition, mini_tiles: i32) -> Self {
        debug_assert!(id > 0);
        let top_mini_tile = &map.mini_tiles[top];
        debug_assert!(top_mini_tile.area_id == id);
        Self {
            id,
            top,
            mini_tiles,
            max_altitude: top_mini_tile.altitude,
            ..Default::default()
        }
    }
}

/// After Areas and ChokePoints, Bases are the third kind of object BWEM automatically computes from Brood War's maps.
/// A Base is essentially a suggested location (intended to be optimal) to put a Command Center, Nexus, or Hatchery.
/// It also provides information on the ressources available, and some statistics.
/// A Base alway belongs to some Area. An Area may contain zero, one or several Bases.
/// Like Areas and ChokePoints, the number and the addresses of Base instances remain unchanged.
///
/// Bases inherit utils::UserData, which provides free-to-use data.
pub struct Base {}

enum Neighbors {
    None,
    One(i16),
    Two(i16, i16),
}

#[derive(Default)]
struct TempAreaInfo {
    valid: bool,
    id: i16,
    top: WalkPosition,
    highest_altitude: Altitude,
    size: i32,
}

impl TempAreaInfo {
    fn new(id: i16, mini_tile: &mut MiniTile, pos: WalkPosition) -> Self {
        let mut result = Self {
            id,
            valid: true,
            top: pos,
            size: 0,
            highest_altitude: mini_tile.altitude,
        };
        result.add(mini_tile);
        result
    }

    fn add(&mut self, mini_tile: &mut MiniTile) {
        debug_assert!(self.valid);
        self.size += 1;
        mini_tile.area_id = self.id;
    }

    fn merge(&mut self, absorbed: &mut TempAreaInfo) {
        debug_assert!(self.valid && absorbed.valid);
        debug_assert!(self.size >= absorbed.size);
        self.size += absorbed.size;
        absorbed.valid = false;
    }
}

#[derive(Default)]
struct Neutral {
    area: Rectangle<TilePosition>,
}

impl Neutral {
    fn next_stacked(&self) -> Option<Neutral> {
        unimplemented!();
    }

    fn set_blocking(&mut self, true_doors: &[WalkPosition]) {
        unimplemented!();
    }

    fn is_static_building(&self) -> bool {
        unimplemented!();
    }
}

#[derive(Default)]
struct Geyser {
    neutral: Neutral,
}

impl From<Unit<'_>> for Geyser {
    fn from(unit: Unit) -> Self {
        Self::default()
    }
}

#[derive(Default)]
struct Mineral {
    neutral: Neutral,
}

impl From<Unit<'_>> for Mineral {
    fn from(unit: Unit) -> Self {
        Self::default()
    }
}

#[derive(Default)]
struct StaticBuilding {
    neutral: Neutral,
}

impl From<Unit<'_>> for StaticBuilding {
    fn from(unit: Unit) -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy)]
pub struct MiniTile {
    area_id: AreaId,
    altitude: Altitude,
}

impl PositionIndexed<8> for MiniTile {}

impl Default for MiniTile {
    fn default() -> Self {
        Self {
            area_id: -1,
            altitude: -1,
        }
    }
}

impl MiniTile {
    fn set_walkable(&mut self, walkable: bool) {
        self.area_id = if walkable { -1 } else { 0 };
        self.altitude = if walkable { -1 } else { 1 };
    }

    fn sea_or_lake(&self) -> bool {
        self.altitude == 1
    }

    fn set_sea(&mut self) {
        debug_assert!(!self.walkable() && self.sea_or_lake());
        self.altitude = 0;
    }

    fn sea(&self) -> bool {
        self.altitude == 0
    }

    fn set_lake(&mut self) {
        debug_assert!(!self.walkable() && self.sea());
        self.altitude = -1;
    }

    fn altitude_missing(&self) -> bool {
        self.altitude == -1
    }

    fn set_altitude(&mut self, altitude: Altitude) {
        debug_assert!(self.altitude_missing() && altitude > 0);
        self.altitude = altitude;
    }

    fn walkable(&self) -> bool {
        self.area_id != 0
    }

    fn set_blocked(&mut self) {
        debug_assert!(self.area_id_missing());
        self.area_id = BLOCKING_CP;
    }

    fn area_id_missing(&self) -> bool {
        self.area_id == -1
    }

    fn replace_area_id(&mut self, area_id: i16) {
        debug_assert!(
            self.area_id > 0 && (area_id >= 1 || area_id <= -2) && self.area_id != area_id,
            "area_id: {}, self.area_id: {}",
            area_id,
            self.area_id
        );
        self.area_id = area_id;
    }
}

#[derive(Default, Clone, Copy)]
pub struct Tile {
    bits: Bits,
    min_altitude: Altitude,
    area_id: AreaId,
}

bitfield! {
    #[derive(Copy, Clone, Default)]
    struct Bits(u8);
    buildable, set_buildable: 0,0;
               ground_height, set_ground_height: 2,1;
                              doodad, set_doodad: 3,3;
}

impl PositionIndexed<32> for Tile {}

impl Tile {
    fn set_buildable(&mut self) {
        self.bits.set_buildable(1);
    }

    fn set_ground_height(&mut self, height: i32) {
        self.bits.set_ground_height(height as u8);
    }

    fn set_doodad(&mut self) {
        self.bits.set_doodad(1);
    }

    fn get_neutral_mut(&mut self) -> Option<&mut Neutral> {
        unimplemented!()
    }

    fn set_min_altitude(&mut self, altitude: Altitude) {
        self.min_altitude = altitude;
    }

    fn set_area_id(&mut self, id: AreaId) {
        self.area_id = id;
    }
}

struct Map {
    walk_area: Rectangle<WalkPosition>,
    tile_area: Rectangle<TilePosition>,
    mini_tiles: Box<[[MiniTile; 256 * 4]]>,
    tiles: Box<[[Tile; 256]]>,
    geysers: Vec<Geyser>,
    minerals: Vec<Mineral>,
    static_buildings: Vec<StaticBuilding>,
    max_altitude: i16,
    starting_locations: Vec<TilePosition>,
    raw_frontier: Vec<((i16, i16), WalkPosition)>,
    areas: Vec<Area>,
    choke_points_matrix: Vec<Vec<Vec<ChokePoint>>>,
}

impl<'a> PositionValidator for Map {
    fn is_valid<const N: i32>(&self, pos: ScaledPosition<N>) -> bool {
        self.tile_area.contains(pos)
    }
}

fn outer_mini_tile_border(area: Rectangle<TilePosition>) -> Vec<WalkPosition> {
    area.to_walk_rect().border()
}

impl Map {
    fn new(game: &Game) -> Self {
        let tile_size = TilePosition::new(game.map_width(), game.map_height());
        let tile_area = Rectangle::new((0, 0), tile_size - (1, 1));
        let walk_area = Rectangle::new((0, 0), tile_size.to_walk_position() - (1, 1));
        let mut result = Self {
            walk_area,
            tile_area,
            mini_tiles: vec![[MiniTile::default(); 256 * 4]; 1 + walk_area.br.y as usize]
                .into_boxed_slice(),
            tiles: vec![[Tile::default(); 256]; 1 + tile_area.br.y as usize].into_boxed_slice(),
            geysers: vec![],
            minerals: vec![],
            static_buildings: vec![],
            max_altitude: 0,
            starting_locations: vec![],
            raw_frontier: vec![],
            areas: vec![],
            choke_points_matrix: vec![],
        };
        result.load_data(game);
        result.decide_seas_or_lakes();
        result.initialize_neutrals(game);
        result.compute_altitude();
        // result.process_blocking_neutrals();
        result.compute_areas();
        result.create_choke_points();
        // result.compute_choke_point_distance_matrix();
        // result.collect_information();
        // result.create_bases();
        result
    }

    fn load_data(&mut self, game: &Game) {
        // Mark unwalkable minitiles (minitiles are walkable by default)
        for wp in self.walk_area {
            if !game.is_walkable(wp) {
                self.mini_tiles[wp].set_walkable(false);

                // For each unwalkable minitile, we also mark its 8 neighbours as not walkable.
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let wp = wp + (dx, dy);
                        if self.is_valid(wp) {
                            self.mini_tiles[wp].set_walkable(false);
                        }
                    }
                }
            }
        }

        // Mark buildable tiles (tiles are unbuildable by default)
        for t in self.tile_area {
            let tile = &mut self.tiles[t];
            if game.is_buildable(t) {
                tile.set_buildable();

                // Ensures buildable ==> walkable:
                for dy in 0..4 {
                    for dx in 0..4 {
                        let wp = t.to_walk_position() + (dx, dy);
                        self.mini_tiles[wp].set_walkable(true);
                    }
                }
            }

            // Add groundHeight and doodad information:
            let bwapi_ground_height = game.get_ground_height(t);
            tile.set_ground_height(bwapi_ground_height / 2);
            if bwapi_ground_height % 2 != 0 {
                tile.set_doodad();
            }
        }
    }

    fn decide_seas_or_lakes(&mut self) {
        for origin in self.walk_area {
            let origin_tile = &mut self.mini_tiles[origin];
            if origin_tile.sea_or_lake() {
                let mut to_search = vec![origin];
                let mut sea_extent = vec![origin];
                origin_tile.set_sea();
                let mut lake_rect = Rectangle::new(origin, origin);
                while let Some(current) = to_search.pop() {
                    lake_rect = lake_rect.resize_to_contain(current);
                    for &delta in &WALK_POSITION_4_DIR {
                        let next = current + delta;
                        if self.is_valid(next) {
                            let next_tile = &mut self.mini_tiles[next];
                            if next_tile.sea_or_lake() {
                                to_search.push(next);
                                if sea_extent.len() <= LAKE_MAX_MINI_TILES {
                                    sea_extent.push(next);
                                }
                                next_tile.set_sea();
                            }
                        }
                    }
                }

                if sea_extent.len() <= LAKE_MAX_MINI_TILES
                    && lake_rect.width() <= LAKE_MAX_WIDTH_IN_MINI_TILES
                    && lake_rect.height() <= LAKE_MAX_WIDTH_IN_MINI_TILES
                    && self.walk_area.shrink(2).envelops(lake_rect)
                {
                    for wp in sea_extent {
                        self.mini_tiles[wp].set_lake();
                    }
                }
            }
        }
    }

    fn initialize_neutrals(&mut self, game: &Game) {
        for n in game.get_static_neutral_units() {
            match n.get_type() {
                UnitType::Resource_Vespene_Geyser => self.geysers.push(n.into()),
                x if x.is_mineral_field() => self.minerals.push(n.into()),
                x if x.is_building() => self.static_buildings.push(n.into()),
                UnitType::Special_Pit_Door | UnitType::Special_Right_Pit_Door => {
                    self.static_buildings.push(n.into())
                }
                _ => (),
            }
        }
    }

    fn compute_altitude(&mut self) {
        const ALTITUDE_SCALE: f32 = 8.; // 8 provides a pixel definition for altitude_t, since altitudes are computed from miniTiles which are 8x8 pixels

        // 1) Fill in and sort DeltasByAscendingAltitude
        let range = self.walk_area.br.x.max(self.walk_area.br.y) / 2 + 3; // should suffice for maps with no Sea.
        let mut deltas_by_ascending_altitude = vec![];

        for dy in 0..=range {
            for dx in dy..=range {
                // Only consider 1/8 of possible deltas. Other ones obtained by symmetry.
                if dx != 0 || dy != 0 {
                    deltas_by_ascending_altitude.push((
                        WalkPosition::new(dx, dy),
                        (0.5 + ((dx * dx + dy * dy) as f32).sqrt() * ALTITUDE_SCALE) as i16,
                    ));
                }
            }
        }

        deltas_by_ascending_altitude.sort_by_key(|e| e.1);

        // 2) Fill in ActiveSeaSideList, which basically contains all the seaside miniTiles (from which altitudes are to be computed)
        //    It also includes extra border-miniTiles which are considered as seaside miniTiles too.
        struct ActiveSeaSide {
            origin: WalkPosition,
            last_altitude_generated: Altitude,
        }

        let mut active_sea_side_list = vec![];

        for wp in self.walk_area.extrude(1) {
            if !self.is_valid(wp) || self.sea_side(wp) {
                active_sea_side_list.push(ActiveSeaSide {
                    origin: wp,
                    last_altitude_generated: 0,
                });
            }
        }
        // 3) Dijkstra's algorithm
        for &(d, altitude) in &deltas_by_ascending_altitude {
            let mut i = 0;
            while i < active_sea_side_list.len() {
                let current = &mut active_sea_side_list[i];
                // optimization : once a seaside miniTile verifies this condition, we can throw it away as it will not generate min altitudes anymore
                if altitude - current.last_altitude_generated >= 2 * ALTITUDE_SCALE as i16 {
                    active_sea_side_list.swap_remove(i);
                } else {
                    for &delta in &[
                        WalkPosition::new(d.x, d.y),
                        WalkPosition::new(-d.x, d.y),
                        WalkPosition::new(d.x, -d.y),
                        WalkPosition::new(-d.x, -d.y),
                        WalkPosition::new(d.y, d.x),
                        WalkPosition::new(-d.y, d.x),
                        WalkPosition::new(d.y, -d.x),
                        WalkPosition::new(-d.y, -d.x),
                    ] {
                        let w = current.origin + delta;
                        if self.is_valid(w) {
                            let mini_tile = &mut self.mini_tiles[w];
                            if mini_tile.altitude_missing() {
                                mini_tile.set_altitude(altitude);
                                self.max_altitude = altitude;
                                current.last_altitude_generated = altitude;
                            }
                        }
                    }
                    i += 1;
                }
            }
        }
    }

    fn sea_side(&self, p: WalkPosition) -> bool {
        if !self.mini_tiles[p].sea() {
            return false;
        }

        WALK_POSITION_4_DIR
            .iter()
            .any(|&delta| self.is_valid(p + delta) && !self.mini_tiles[p + delta].sea())
    }

    fn process_blocking_neutrals(&mut self) {
        for p_candidate in self
            .static_buildings
            .iter()
            .map(|s| &s.neutral)
            .chain(self.minerals.iter().map(|m| &m.neutral))
        {
            if p_candidate.next_stacked().is_none() {
                // in the case where several neutrals are stacked, we only consider the top one
                // 1)  Retreave the Border: the outer border of pCandidate
                let mut border = outer_mini_tile_border(p_candidate.area);

                // 2)  Find the doors in Border: one door for each connected set of walkable, neighbouring miniTiles.
                //     The searched connected miniTiles all have to be next to some lake or some static building, though they can't be part of one.
                let mut doors = vec![];
                while let Some(door) = border.pop() {
                    doors.push(door);
                    let mut to_visit = vec![door];
                    let mut visited = vec![door];
                    while let Some(current) = to_visit.pop() {
                        for &delta in &WALK_POSITION_4_DIR {
                            let next = current + delta;
                            if self.is_valid(next)
                                && !visited.contains(&next)
                                && self.mini_tiles[next].walkable()
                                && self.tiles[next.to_tile_position()]
                                    .get_neutral_mut()
                                    .is_none()
                                && self.adjouns_8_some_lake_or_neutral(next)
                            {
                                to_visit.push(next);
                                visited.push(next);
                            }
                        }
                    }
                    doors.retain(|w| !visited.contains(w));
                }

                // 3)  If at least 2 doors, find the true doors in Border: a true door is a door that gives onto an area big enough
                if doors.len() >= 2 {
                    let mut true_doors = vec![];
                    for &door in &doors {
                        let mut to_visit = vec![door];
                        let mut visited = vec![door];
                        let limit = if p_candidate.is_static_building() {
                            10
                        } else {
                            400
                        };
                        while !to_visit.is_empty() && visited.len() < limit {
                            let current = to_visit.pop().unwrap();
                            for &delta in &WALK_POSITION_4_DIR {
                                let next = current + delta;
                                if self.is_valid(next)
                                    && !visited.contains(&next)
                                    && self.mini_tiles[next].walkable()
                                    && self.tiles[next.to_tile_position()]
                                        .get_neutral_mut()
                                        .is_none()
                                {
                                    to_visit.push(next);
                                    visited.push(next);
                                }
                            }
                        }

                        // 4)  If at least 2 true doors, pCandidate is a blocking static building
                        if visited.len() >= 2 {
                            // Marks pCandidate (and any Neutral stacked with it) as blocking.
                            let mut p_neutral = self.tiles[p_candidate.area.tl].get_neutral_mut();
                            while let Some(ref mut neutral) = p_neutral {
                                neutral.set_blocking(&true_doors);
                            }

                            // Marks all the miniTiles of pCandidate as blocked.
                            // This way, areas at TrueDoors won't merge together.
                            for wp in p_candidate.area.to_walk_rect() {
                                let mini_tile = &mut self.mini_tiles[wp];
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

    fn adjouns_8_some_lake_or_neutral(&self, pos: WalkPosition) -> bool {
        unimplemented!();
    }

    fn compute_areas(&mut self) {
        let mut mini_tiles_by_descending_altitude: Vec<_> = self
            .walk_area
            .into_iter()
            .filter(|&w| self.mini_tiles[w].area_id_missing())
            .collect();
        mini_tiles_by_descending_altitude.sort_unstable_by_key(|&a| -self.mini_tiles[a].altitude);

        let temp_area_list = self.compute_temp_areas(mini_tiles_by_descending_altitude);

        self.create_areas(temp_area_list);

        self.set_area_id_in_tiles();
    }

    fn compute_temp_areas(
        &mut self,
        mini_tiles_by_descending_altitude: Vec<WalkPosition>,
    ) -> Vec<TempAreaInfo> {
        let mut temp_area_list = vec![Default::default()]; // TempAreaList[0] left unused, as AreaIds are > 0

        let mut map_area_pair_counter = HashMap::new();
        for &pos in &mini_tiles_by_descending_altitude {
            let neighboring_areas = self.find_neighboring_areas(pos);
            let cur = &mut self.mini_tiles[pos];
            match neighboring_areas {
                Neighbors::None => {
                    // no neighboring area : creates a new area
                    temp_area_list.push(TempAreaInfo::new(temp_area_list.len() as i16, cur, pos))
                }
                Neighbors::One(area_id) => temp_area_list[area_id as usize].add(cur), // one neighboring area : adds cur to the existing area
                Neighbors::Two(a, b) => {
                    // two neighboring areas : adds cur to one of them  &  possible merging
                    let (mut smaller, mut bigger) = (a as usize, b as usize);
                    if temp_area_list[smaller].size > temp_area_list[bigger].size {
                        std::mem::swap(&mut smaller, &mut bigger);
                    }

                    // Condition for the neighboring areas to merge:
                    if temp_area_list[smaller].size < 80
                        || temp_area_list[smaller].highest_altitude < 80
                        || cur.altitude * 10 >= temp_area_list[bigger].highest_altitude * 9
                        || cur.altitude * 10 >= temp_area_list[smaller].highest_altitude * 9
                        || self.starting_locations.iter().any(|&starting_loc| {
                            pos.to_tile_position()
                                .distance_squared(starting_loc + (2, 1))
                                <= 9
                        })
                    {
                        // adds cur to the absorbing area:
                        temp_area_list[bigger].add(cur);

                        // merges the two neighboring areas:
                        self.replace_area_ids(temp_area_list[smaller].top, bigger as i16);
                        let (big_area, small_area) = pick(&mut temp_area_list, bigger, smaller);
                        big_area.merge(small_area);
                    } else {
                        // no merge : cur starts or continues the frontier between the two neighboring areas
                        // adds cur to the chosen Area:
                        let (mut a, mut b) = (smaller, bigger);
                        if a > b {
                            std::mem::swap(&mut a, &mut b);
                        }
                        let v = map_area_pair_counter.entry((a, b)).or_insert(0);
                        let neighboring_area = if *v % 2 == 0 { a } else { b };
                        *v += 1;

                        temp_area_list[neighboring_area].add(cur);
                        self.raw_frontier
                            .push(((smaller as i16, bigger as i16), pos));
                    }
                }
            }
        }
        self.raw_frontier.retain(|((a, b), _)| a != b);
        temp_area_list
    }

    fn replace_area_ids(&mut self, p: WalkPosition, new_area_id: i16) {
        let origin = &mut self.mini_tiles[p];
        let old_area_id = origin.area_id;
        origin.replace_area_id(new_area_id);

        let mut to_search = vec![p];
        while let Some(current) = to_search.pop() {
            for &delta in &WALK_POSITION_4_DIR {
                let next = current + delta;
                if self.is_valid(next) {
                    let next_tile = &mut self.mini_tiles[next];
                    if next_tile.area_id == old_area_id {
                        to_search.push(next);
                        next_tile.replace_area_id(new_area_id);
                    }
                }
            }
        }

        // also replaces references of oldAreaId by newAreaId in m_RawFrontier:
        if new_area_id > 0 {
            for f in &mut self.raw_frontier {
                if f.0 .0 == old_area_id {
                    f.0 .0 = new_area_id;
                }
                if f.0 .1 == old_area_id {
                    f.0 .1 = new_area_id;
                }
            }
        }
    }

    fn find_neighboring_areas(&self, p: WalkPosition) -> Neighbors {
        let mut result = Neighbors::None;
        for &delta in &WALK_POSITION_4_DIR {
            if self.is_valid(p + delta) {
                let area_id = self.mini_tiles[p + delta].area_id;
                if area_id > 0 {
                    result = match result {
                        Neighbors::None => Neighbors::One(area_id),
                        Neighbors::One(x) if x != area_id => Neighbors::Two(x, area_id),
                        Neighbors::Two(x, y) if x != area_id && area_id < y => {
                            Neighbors::Two(x, area_id)
                        }
                        x => x,
                    };
                }
            }
        }
        result
    }

    fn create_areas(&mut self, temp_area_list: Vec<TempAreaInfo>) {
        type PairTopSize = (WalkPosition, i32);
        let mut areas_list = vec![];

        let mut new_area_id = 1;
        let mut new_tiny_area_id = -2;

        for temp_area in &temp_area_list {
            if temp_area.valid {
                if temp_area.size >= AREA_MIN_MINI_TILES {
                    debug_assert!(new_area_id <= temp_area.id);
                    if new_area_id != temp_area.id {
                        self.replace_area_ids(temp_area.top, new_area_id);
                    }
                    areas_list.push((temp_area.top, temp_area.size));
                    new_area_id += 1;
                } else {
                    self.replace_area_ids(temp_area.top, new_tiny_area_id);
                    new_tiny_area_id -= 1;
                }
            }
        }

        self.areas = areas_list
            .iter()
            .enumerate()
            .map(|(id, &(top, mini_tiles))| Area::new(self, id as AreaId + 1, top, mini_tiles))
            .collect();
    }

    fn set_area_id_in_tile(&mut self, t: TilePosition) {
        let tile = &mut self.tiles[t];
        debug_assert!(tile.area_id == 0); // initialized to 0

        let tl = t.to_walk_position();
        for dy in 0..4 {
            for dx in 0..4 {
                let id = self.mini_tiles[tl + (dx, dy)].area_id;
                if tile.area_id == 0 {
                    tile.set_area_id(id);
                } else if tile.area_id != id {
                    tile.set_area_id(-1);
                    return;
                }
            }
        }
    }

    fn set_altitude_in_tile(&mut self, t: TilePosition) {
        let mut min_altitude = Altitude::MAX;

        let tl = t.to_walk_position();
        for dy in 0..4 {
            for dx in 0..4 {
                let altitude = self.mini_tiles[tl + (dx, dy)].altitude;
                if altitude < min_altitude {
                    min_altitude = altitude;
                }
            }
        }

        self.tiles[t].set_min_altitude(min_altitude);
    }

    fn set_area_id_in_tiles(&mut self) {
        for tp in self.tile_area {
            self.set_area_id_in_tile(tp);
            self.set_altitude_in_tile(tp);
        }
    }

    fn create_choke_points(&mut self) {
        let mut new_index = 0;

        let pseudo_choke_points_to_create = 1;
        let mut choke_points_matrix: Vec<Vec<Vec<ChokePoint>>> = vec![];
        // 1) Size the matrix
        for id in 0..=self.areas.len() {
            choke_points_matrix.push(vec![Default::default(); id]);
        }

        // 2) Dispatch the global raw frontier between all the relevant pairs of Areas:
        let mut raw_frontier_by_area_pair = HashMap::new();
        for ((mut a, mut b), wp) in &self.raw_frontier {
            if a > b {
                std::mem::swap(&mut a, &mut b)
            }
            debug_assert!(a >= 1 && b as usize <= self.areas.len());
            raw_frontier_by_area_pair
                .entry((a, b))
                .or_insert_with(Vec::new)
                .push(wp);
        }

        // 3) For each pair of Areas (A, B):
        for (&(a, b), raw_frontier_ab) in &raw_frontier_by_area_pair {
            // Because our dispatching preserved order,
            // and because Map::m_RawFrontier was populated in descending order of the altitude (see Map::ComputeAreas),
            // we know that RawFrontierAB is also ordered the same way, but let's check it:
            debug_assert!({
                let altitudes: Vec<_> = raw_frontier_ab
                    .iter()
                    .map(|&&wp| self.mini_tiles[wp].altitude)
                    .collect();
                // altitudes.is_sorted();
                true
            });

            // 3.1) Use that information to efficiently cluster RawFrontierAB in one or several chokepoints.
            //    Each cluster will be populated starting with the center of a chokepoint (max altitude)
            //    and finishing with the ends (min altitude).
            let cluster_min_dist = (LAKE_MAX_MINI_TILES as f32).sqrt() as u32;
            let mut clusters: Vec<VecDeque<WalkPosition>> = vec![];
            for &&w in raw_frontier_ab {
                let mut added = false;
                for cluster in &mut clusters {
                    let dist_to_front = cluster.front().unwrap().chebyshev_distance(w);
                    let dist_to_back = cluster.back().unwrap().chebyshev_distance(w);
                    if dist_to_front <= cluster_min_dist || dist_to_back <= cluster_min_dist {
                        if dist_to_front < dist_to_back {
                            cluster.push_front(w);
                        } else {
                            cluster.push_back(w);
                        }
                        added = true;
                        break;
                    }
                }
                if !added {
                    let mut new_cluster = VecDeque::new();
                    new_cluster.push_back(w);
                    clusters.push(new_cluster);
                }
            }

            // 3.2) Create one Chokepoint for each cluster:
            let choke_points = &mut choke_points_matrix[b as usize][a as usize];
            choke_points.reserve(clusters.len() + pseudo_choke_points_to_create);
            for cluster in clusters.iter_mut() {
                choke_points.push(ChokePoint::new(
                    self,
                    new_index,
                    a,
                    b,
                    cluster.make_contiguous(),
                ));
            }
        }
        self.choke_points_matrix = choke_points_matrix;
    }

    fn compute_choke_point_distance_matrix(&mut self) {
        unimplemented!()
    }
    fn collect_information(&mut self) {
        unimplemented!()
    }
    fn create_bases(&mut self) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tes {
    use super::*;
    use crate::{command::Commands, game::*};
    use image::*;
    use inflate::inflate_bytes_zlib;
    use shm::Shm;
    use std::cell::RefCell;
    use std::fs::*;
    use std::time::Instant;

    #[test]
    fn bits_should_not_bleed_over() {
        let mut bits = Bits::default();
        assert_eq!(bits.buildable(), 0);
        assert_eq!(bits.ground_height(), 0);
        assert_eq!(bits.doodad(), 0);

        bits.set_doodad(1);
        assert_eq!(bits.buildable(), 0);
        assert_eq!(bits.ground_height(), 0);
        assert_eq!(bits.doodad(), 1);

        let mut bits = Bits::default();
        bits.set_ground_height(3);
        assert_eq!(bits.buildable(), 0);
        assert_eq!(bits.ground_height(), 3);
        assert_eq!(bits.doodad(), 0);

        let mut bits = Bits::default();
        bits.set_buildable(1);
        assert_eq!(bits.buildable(), 1);
        assert_eq!(bits.ground_height(), 0);
        assert_eq!(bits.doodad(), 0);
    }

    #[test]
    fn test_maps() {
        for entry in read_dir("resources/test").unwrap() {
            let entry = entry.unwrap();
            println!("Reading map {:?}", entry.path());
            let data = read(entry.path()).unwrap();
            let mut inflated = inflate_bytes_zlib(&data).unwrap();
            let shm = unsafe { Shm::from_slice(inflated.as_mut_slice().into()) };
            let mut game_context = GameContext::new(shm);
            let commands = RefCell::new(Commands::new());
            game_context.match_start();
            game_context.with_frame(&commands, |game| {
                let timer = Instant::now();
                let map = Map::new(game);
                let mut img: RgbImage =
                    ImageBuffer::new(4 * game.map_width() as u32, 4 * game.map_height() as u32);
                for (y, row) in map
                    .mini_tiles
                    .iter()
                    .enumerate()
                    .take(game.map_height() as usize * 4)
                {
                    for (x, alt) in row.iter().enumerate().take(game.map_width() as usize * 4) {
                        let a = alt.altitude;
                        if a < 0 {
                            img.put_pixel(x as u32, y as u32, Rgb([0, 0, 255 - (-a / 2) as u8]));
                        } else if alt.area_id > 0 {
                            img.put_pixel(
                                x as u32,
                                y as u32,
                                Rgb([255 - (a / 2) as u8, (37 * alt.area_id % 255) as u8, 0]),
                            );
                        }
                    }
                }
                for (_, f) in map.raw_frontier {
                    img.put_pixel(f.x as u32, f.y as u32, Rgb([255, 255, 255]));
                }

                img.save(format!(
                    "{}.png",
                    entry.path().file_name().unwrap().to_string_lossy()
                ))
                .unwrap();
            });
        }
        panic!();
    }
}
