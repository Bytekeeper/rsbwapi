use super::{area::*, defs::*, neutral::*};
use std::cell::RefCell;
use std::rc::Rc;

///////////////////////////////////////////////////////////////////////////////////////////////
///                                                                                          //
///                                  class MiniTile                                                                                   
///                                                                                          //
///////////////////////////////////////////////////////////////////////////////////////////////
///                                                                                                                                   
/// Corresponds to BWAPI/Starcraft's concept of minitile (8x8 pixels).                                                                
/// MiniTiles are accessed using WalkPositions (Cf. Map::GetMiniTile).                                                                
/// A Map holds Map::WalkSize().x * Map::WalkSize().y MiniTiles as its "MiniTile map".                                                
/// A MiniTile contains essentialy 3 informations:                                                                                    
///      - its Walkability                                                                                                            
///      - its altitude (distance from the nearest non walkable MiniTile, except those which are part of small enough zones (lakes))  
///      - the id of the Area it is part of, if ever.                                                                                 
/// The whole process of analysis of a Map relies on the walkability information                                                      
/// from which are derived successively : altitudes, Areas, ChokePoints.
pub struct MiniTile {
    altitude: Altitude,
    area_id: AreaId,
}

///////////////////////////////////////////////////////////////////////////////////////////////
///                                                                                          //
///                                  class Tile
///                                                                                          //
///////////////////////////////////////////////////////////////////////////////////////////////
///
/// Corresponds to BWAPI/Starcraft's concept of tile (32x32 pixels).
/// Tiles are accessed using TilePositions (Cf. Map::GetTile).
/// A Map holds Map::Size().x * Map::Size().y Tiles as its "Tile map".
///
/// It should be noted that a Tile exactly overlaps 4 x 4 MiniTiles.
/// As there are 16 times as many MiniTiles as Tiles, we allow a Tiles to contain more data than MiniTiles.
/// As a consequence, Tiles should be preferred over MiniTiles, for efficiency.
/// The use of Tiles is further facilitated by some functions like Tile::AreaId or Tile::MinAltitude
/// which somewhat aggregate the MiniTile's corresponding information
///
/// Tiles inherit utils::Markable, which provides marking ability
/// Tiles inherit utils::UserData, which provides free-to-use data.
pub struct Tile {
    p_neutral: Option<Rc<RefCell<Neutral>>>,
    min_altitude: Altitude,
    area_id: AreaId,
    internal_data: isize,
    bits: Bits,
}

/// TODO: These are actual bits in the original implementation!
pub struct Bits {
    buildable: bool,
    ground_height: u8,
    doodad: bool,
}

impl Tile {
    /// Corresponds to BWAPI::isBuildable
    /// Note: BWEM enforces the relation buildable ==> walkable (Cf. MiniTile::Walkable)
    pub fn buildable(&self) -> bool {
        self.bits.buildable
    }

    /// Tile::AreaId() somewhat aggregates the MiniTile::AreaId() values of the 4 x 4 sub-MiniTiles.
    /// Let S be the set of MiniTile::AreaId() values for each walkable MiniTile in this Tile.
    /// If empty(S), returns 0. Note: in this case, no contained MiniTile is walkable, so all of them have their AreaId() == 0.
    /// If S = {a}, returns a (whether positive or negative).
    /// If size(S) > 1 returns -1 (note that -1 is never returned by MiniTile::AreaId()).
    pub fn area_id(&self) -> AreaId {
        self.area_id
    }

    /// Tile::MinAltitude() somewhat aggregates the MiniTile::Altitude() values of the 4 x 4 sub-MiniTiles.
    /// Returns the minimum value.
    pub fn min_altitude(&self) -> Altitude {
        self.min_altitude
    }

    /// Tells if at least one of the sub-MiniTiles is Walkable.
    pub fn walkable(&self) -> bool {
        self.area_id != 0
    }

    /// Tells if at least one of the sub-MiniTiles is a Terrain-MiniTile.
    pub fn terrain(&self) -> bool {
        self.walkable()
    }

    /// 0: lower ground    1: high ground    2: very high ground
    /// Corresponds to BWAPI::getGroundHeight / 2
    pub fn ground_height(&self) -> isize {
        self.bits.ground_height as isize
    }

    /// Tells if this Tile is part of a doodad.
    /// Corresponds to BWAPI::getGroundHeight % 2
    pub fn doodad(&self) -> bool {
        self.bits.doodad
    }

    /// If any Neutral occupies this Tile, returns it (note that all the Tiles it occupies will then return it).
    /// Otherwise, returns nullptr.
    /// Neutrals are Minerals, Geysers and StaticBuildings (Cf. Neutral).
    /// In some maps (e.g. Benzene.scx), several Neutrals are stacked at the same location.
    /// In this case, only the "bottom" one is returned, while the other ones can be accessed using Neutral::NextStacked().
    /// Because Neutrals never move on the Map, the returned value is guaranteed to remain the same, unless some Neutral
    /// is destroyed and BWEM is informed of that by a call of Map::OnMineralDestroyed(BWAPI::Unit u) for exemple. In such a case,
    /// BWEM automatically updates the data by deleting the Neutral instance and clearing any reference to it such as the one
    /// returned by Tile::GetNeutral(). In case of stacked Neutrals, the next one is then returned.
    pub fn get_neutral(&self) -> Option<Rc<RefCell<Neutral>>> {
        self.p_neutral.clone()
    }

    /// Returns the number of Neutrals that occupy this Tile (Cf. GetNeutral).
    pub fn stacked_neutrals(&self) -> isize {
        unimplemented!()
    }

    ///      Details: The functions below are used by the BWEM's internals

    pub fn set_buildable(&mut self) {
        self.bits.buildable = true;
    }

    pub fn set_ground_height(&mut self, h: isize) {
        debug_assert!((0 <= h) && (h <= 2));
        self.bits.ground_height = h as u8;
    }

    pub fn set_doodad(&mut self) {
        self.bits.doodad = true;
    }

    pub fn add_neutral(&mut self, p_neutral: Rc<RefCell<Neutral>>) {
        debug_assert!(self.p_neutral.is_none());
        self.p_neutral = Some(p_neutral);
    }

    pub fn set_area_id(&mut self, id: AreaId) {
        debug_assert!((id == -1) || self.area_id == 0 && id != 0);
        self.area_id = id;
    }

    pub fn reset_area_id(&mut self) {
        self.area_id = 0;
    }

    pub fn set_min_altitude(&mut self, a: Altitude) {
        debug_assert!(a >= 0);
        self.min_altitude = a;
    }

    pub fn remove_neutral(&mut self, p_neutral: Rc<RefCell<Neutral>>) {
        debug_assert!(self.p_neutral == Some(p_neutral));
        unimplemented!()
    }
    // void                            RemoveNeutral(Neutral * pNeutral){ bwem_assert(pNeutral && (m_pNeutral == pNeutral)); utils::unused(pNeutral); m_pNeutral = nullptr; }
    /*
    int                                     InternalData() const                    { return m_internalData; }
    void                            SetInternalData(int data) const { m_internalData = data; }
    */
}
