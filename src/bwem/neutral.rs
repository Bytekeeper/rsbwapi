use super::{area::*, map::*};
use crate::*;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

pub struct UnitMemo {
    id: UnitId,
}

pub trait Neutral {
    fn as_neutral(&self) -> Neutral;

    // If this Neutral is a Mineral, returns a typed pointer to this Mineral.
    // Otherwise, returns nullptr.
    fn is_mineral(&self) -> Option<Mineral> {
        None
    }

    /// If this Neutral is a Geyser, returns a typed pointer to this Geyser.
    /// Otherwise, returns nullptr.
    fn is_geyser(&self) -> Option<Geyser> {
        None
    }

    /// If this Neutral is a StaticBuilding, returns a typed pointer to this StaticBuilding.
    /// Otherwise, returns nullptr.
    fn is_static_building(&self) -> Option<StaticBuilding> {
        None
    }

    /// Returns the BWAPI::Unit this Neutral is wrapping around.
    fn unit_memo(&self) -> &UnitMemo {
        &self.bwapi_unit
    }

    /// Returns the BWAPI::UnitType of the BWAPI::Unit this Neutral is wrapping around.
    fn unit_type(&self) -> UnitType {
        self.bwapi_type
    }

    /// Returns the center of this Neutral, in pixels (same as Unit()->getInitialPosition()).
    fn pos(&self) -> Position {
        self.pos
    }

    /// Returns the top left Tile position of this Neutral (same as Unit()->getInitialTilePosition()).
    fn top_left(&self) -> TilePosition {
        self.top_left
    }

    /// Returns the bottom right Tile position of this Neutral
    fn bottom_right(&self) -> TilePosition {
        unimplemented!()
    }

    /// Returns the size of this Neutral, in Tiles (same as Type()->tileSize())
    fn size(&self) -> TilePosition {
        self.as_neutral().size
    }

    /// Tells whether this Neutral is blocking some ChokePoint.
    /// This applies to Minerals and StaticBuildings only.
    /// For each blocking Neutral, a pseudo ChokePoint (which is Blocked()) is created on top of it,
    /// with the exception of stacked blocking Neutrals for which only one pseudo ChokePoint is created.
    /// Cf. definition of pseudo ChokePoints in class ChokePoint comment.
    /// Cf. ChokePoint::BlockingNeutral and ChokePoint::Blocked.
    fn blocking(&self) -> bool {
        !self.blocked_areas.is_empty()
    }

    /// If Blocking() == true, returns the set of Areas blocked by this Neutral.
    fn blocked_areas(&self) -> Vec<Rc<RefCell<Area>>> {
        unimplemented!()
    }

    /// Returns the next Neutral stacked over this Neutral, if ever.
    /// To iterate through the whole stack, one can use the following:
    /// for (const Neutral * n = Map::GetTile(TopLeft()).GetNeutral() ; n ; n = n->NextStacked())
    fn next_stacked(&self) -> Option<Neutral> {
        self.next_stacked
    }

    fn put_on_tiles(&self) {
        let cloned = neutral.as_neutral();
        let neutral_ = neutral.borrow();
        let neutral = neutral_.data();
        debug_assert!(neutral.next_stacked().is_none());

        let mut map = neutral.map.borrow_mut();
        for dy in 0..neutral.size.y {
            for dx in 0..neutral.size.x {
                let tile = map.get_tile_mut(neutral.top_left + (dx, dy));
                if let Some(top) = tile.get_neutral() {
                    debug_assert!(*neutral_ != *top.borrow());
                    let top = last_stacked(top);
                    debug_assert!(*neutral_ != *top.borrow());
                    debug_assert!(Neutral::is_geyser(&*top.borrow()).is_none());
                } else {
                    tile.add_neutral(cloned);
                    return;
                }
            }
        }
    }

    /// Returns the last Neutral stacked over this Neutral, if ever.
    fn last_stacked(&self) -> Neutral {
        let mut top = self.as_neutral();
        while let Some(next) = top.next_stacked() {
            top = next;
        }
        top
    }
}

impl NeutralLike for Neutral {
    fn as_neutral(&self) -> Neutral {
        self.clone()
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////
///                                                                                          //
/// class Neutral
///                                                                                          //
///////////////////////////////////////////////////////////////////////////////////////////////
///
/// Neutral is the abstract base class for a small hierarchy of wrappers around some BWAPI::Units
/// The units concerned are the Ressources (Minerals and Geysers) and the static Buildings.
/// Stacked Neutrals are supported, provided they share the same type at the same location.
///
neutral_data!();

macro_rules! neutral_data {
    ($n:ident, $( $i: ident : $t: ty)) => {{
        struct $n {
            bwapi_unit: UnitMemo,
            bwapi_type: UnitType,
            pos: Position,
            top_left: TilePosition,
            size: TilePosition,
            map: Rc<RefCell<Map>>,
            next_stacked: Option<N>,
            blocked_areas: Vec<WalkPosition>,
            $($i : $t)*
        }

        impl Neutral for $n {

        }
    }};
}

// impl<T: NeutralLike> PartialEq<T> for dyn NeutralLike {
//     fn eq(&self, other: &dyn NeutralLike) -> bool {
//         self.as_neutral().unit_memo().id == other.as_neutral().unit_memo().id
//     }
// }

///////////////////////////////////////////////////////////////////////////////////////////////
///                                                                                          //
/// class Ressource
///                                                                                          //
///////////////////////////////////////////////////////////////////////////////////////////////
///
/// A Ressource is either a Mineral or a Geyser
pub struct ResourceData {
    initial_amount: isize,
}

impl ResourceData {
    /// Returns the initial amount of ressources for this Ressource (same as Unit()->getInitialResources).
    fn initial_amount(&self) -> isize {
        self.initial_amount
    }

    /// Returns the current amount of ressources for this Ressource (same as Unit()->getResources).
    fn amount(&self) -> usize {
        unimplemented!()
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////
///                                                                                          //
/// class Mineral
///                                                                                          //
///////////////////////////////////////////////////////////////////////////////////////////////
///
/// Minerals Correspond to the units in BWAPI::getStaticNeutralUnits() for which getType().isMineralField(),

pub struct MineralData {}

// impl Mineral for MineralData {}
///////////////////////////////////////////////////////////////////////////////////////////////
///                                                                                          //
/// class Geyser
///                                                                                          //
///////////////////////////////////////////////////////////////////////////////////////////////
///
/// Geysers Correspond to the units in BWAPI::getStaticNeutralUnits() for which getType() == Resource_Vespene_Geyser
pub struct GeyserData {}

// impl Geyser for GeyserData {}

///////////////////////////////////////////////////////////////////////////////////////////////
///                                                                                          //
/// class StaticBuilding
///                                                                                          //
///////////////////////////////////////////////////////////////////////////////////////////////
///
/// StaticBuildings Correspond to the units in BWAPI::getStaticNeutralUnits() for which getType().isSpecialBuilding
/// StaticBuilding also wrappers some special units like Special_Pit_Door.
pub struct StaticBuildingData {}
