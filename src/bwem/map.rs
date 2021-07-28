use super::{area::*, cp::*, defs::*, neutral::*, tiles::*};
use crate::*;
use std::cell::RefCell;
use std::rc::Rc;

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
    size: isize,
    Size: TilePosition,
    walk_size: isize,
    Walk_size: WalkPosition,
    center: Position,
    tiles: Vec<Tile>,
    mini_tiles: Vec<MiniTile>,
}

impl Map {
    /// This has to be called before any other function is called.
    /// A good place to do this is in ExampleAIModule::onStart()
    pub fn initialize(&mut self, game: &Game) {
        unimplemented!()
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
        unimplemented!()
    }

    /// Enables the automatic path update (Cf. AutomaticPathUpdate()).
    /// One might NOT want to call this function, in order to make the accessibility between Areas remain the same throughout the game.
    /// Even in this case, one should keep calling OnMineralDestroyed and OnStaticBuildingDestroyed.
    pub fn enable_automatic_path_analysis(&self) {
        unimplemented!()
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

    /// Returns a Tile or a MiniTile, given its position.
    /// Provided as a support of generic algorithms.
    pub fn get_t_tile<P, T: TileOfPosition<P>>(&self, p: P) -> T::Tile {
        unimplemented!()
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
    pub fn minerals(&self) -> &[Rc<RefCell<Mineral>>] {
        unimplemented!()
    }

    /// Returns a reference to the Geysers (Cf. Geyser).
    pub fn geysers(&self) -> &[Rc<RefCell<Geyser>>] {
        unimplemented!()
    }

    /// Returns a reference to the StaticBuildings (Cf. StaticBuilding).
    pub fn static_buildings(&self) -> &[Rc<RefCell<StaticBuilding>>] {
        unimplemented!()
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
    pub fn breadth_first_search<T, P1, P2>(
        &self,
        start: T,
        find_cond: P1,
        visit_cond: P2,
        connect8: bool,
    ) -> T {
        unimplemented!()
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
