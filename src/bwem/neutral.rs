use super::{area::*, map::*};
use crate::*;
use std::cell::RefCell;
use std::rc::Rc;

struct NeutralData {
    unit_id: UnitId,
    unit_type: UnitType,
    pos: Position,
    top_left: TilePosition,
    bottom_right: TilePosition,
    size: TilePosition,
    blocked_areas: Vec<WalkPosition>,
}

pub trait AsNeutral {
    // If this Neutral is a Mineral, returns a typed pointer to this Mineral.
    // Otherwise, returns nullptr.
    fn is_mineral(&self) -> Option<&Mineral> {
        None
    }

    /// If this Neutral is a Geyser, returns a typed pointer to this Geyser.
    /// Otherwise, returns nullptr.
    fn is_geyser(&self) -> Option<&Geyser> {
        None
    }

    /// If this Neutral is a StaticBuilding, returns a typed pointer to this StaticBuilding.
    /// Otherwise, returns nullptr.
    fn is_static_building(&self) -> Option<&StaticBuilding> {
        None
    }
}

pub trait Neutral: AsNeutral {
    /// Returns the BWAPI::Unit this Neutral is wrapping around.
    fn unit_id(&self) -> UnitId;

    /// Returns the BWAPI::UnitType of the BWAPI::Unit this Neutral is wrapping around.
    fn unit_type(&self) -> UnitType;

    /// Returns the center of this Neutral, in pixels (same as Unit()->getInitialPosition()).
    fn pos(&self) -> Position;

    /// Returns the top left Tile position of this Neutral (same as Unit()->getInitialTilePosition()).
    fn top_left(&self) -> TilePosition;

    /// Returns the bottom right Tile position of this Neutral
    fn bottom_right(&self) -> TilePosition;

    /// Returns the size of this Neutral, in Tiles (same as Type()->tileSize())
    fn size(&self) -> TilePosition;

    /// Tells whether this Neutral is blocking some ChokePoint.
    /// This applies to Minerals and StaticBuildings only.
    /// For each blocking Neutral, a pseudo ChokePoint (which is Blocked()) is created on top of it,
    /// with the exception of stacked blocking Neutrals for which only one pseudo ChokePoint is created.
    /// Cf. definition of pseudo ChokePoints in class ChokePoint comment.
    /// Cf. ChokePoint::BlockingNeutral and ChokePoint::Blocked.
    fn blocking(&self) -> bool;

    fn set_blocking(&mut self, blocked_areas: &[WalkPosition]);

    /// If Blocking() == true, returns the set of Areas blocked by this Neutral.
    fn blocked_areas(&self) -> Vec<Rc<RefCell<Area>>>;

    /// Returns the next Neutral stacked over this Neutral, if ever.
    /// To iterate through the whole stack, one can use the following:
    /// for (const Neutral * n = Map::GetTile(TopLeft()).GetNeutral() ; n ; n = n->NextStacked())
    fn next_stacked(&self) -> *mut dyn Neutral;

    fn put_on_tiles(&self);

    /// Returns the last Neutral stacked over this Neutral, if ever.
    fn last_stacked(&self) -> Rc<dyn Neutral>;
}

macro_rules! neutral_delegate {
    ($l:ident) => {
        impl Neutral for $l {
            /// Returns the BWAPI::Unit this Neutral is wrapping around.
            fn unit_id(&self) -> UnitId {
                self.neutral.unit_id
            }

            /// Returns the BWAPI::UnitType of the BWAPI::Unit this Neutral is wrapping around.
            fn unit_type(&self) -> UnitType {
                self.neutral.unit_type
            }

            /// Returns the center of this Neutral, in pixels (same as Unit()->getInitialPosition()).
            fn pos(&self) -> Position {
                self.neutral.pos
            }

            /// Returns the top left Tile position of this Neutral (same as Unit()->getInitialTilePosition()).
            fn top_left(&self) -> TilePosition {
                self.neutral.top_left
            }

            /// Returns the bottom right Tile position of this Neutral
            fn bottom_right(&self) -> TilePosition {
                self.neutral.bottom_right
            }

            /// Returns the size of this Neutral, in Tiles (same as Type()->tileSize())
            fn size(&self) -> TilePosition {
                self.neutral.size
            }

            /// Tells whether this Neutral is blocking some ChokePoint.
            /// This applies to Minerals and StaticBuildings only.
            /// For each blocking Neutral, a pseudo ChokePoint (which is Blocked()) is created on top of it,
            /// with the exception of stacked blocking Neutrals for which only one pseudo ChokePoint is created.
            /// Cf. definition of pseudo ChokePoints in class ChokePoint comment.
            /// Cf. ChokePoint::BlockingNeutral and ChokePoint::Blocked.
            fn blocking(&self) -> bool {
                unimplemented!()
            }

            fn set_blocking(&mut self, blocked_areas: &[WalkPosition]) {
                self.neutral.blocked_areas = blocked_areas.to_vec();
            }

            /// If Blocking() == true, returns the set of Areas blocked by this Neutral.
            fn blocked_areas(&self) -> Vec<Rc<RefCell<Area>>> {
                unimplemented!()
            }

            /// Returns the next Neutral stacked over this Neutral, if ever.
            /// To iterate through the whole stack, one can use the following:
            /// for (const Neutral * n = Map::GetTile(TopLeft()).GetNeutral() ; n ; n = n->NextStacked())
            fn next_stacked(&self) -> *mut dyn Neutral {
                unimplemented!()
            }

            fn put_on_tiles(&self) {
                unimplemented!()
            }

            /// Returns the last Neutral stacked over this Neutral, if ever.
            fn last_stacked(&self) -> Rc<dyn Neutral> {
                unimplemented!()
            }
        }
    };
}

///////////////////////////////////////////////////////////////////////////////////////////////
///                                                                                          //
/// class Ressource
///                                                                                          //
///////////////////////////////////////////////////////////////////////////////////////////////
///
/// A Ressource is either a Mineral or a Geyser
pub struct ResourceData {
    initial_amount: isize,
    neutral: NeutralData,
}

pub trait Resource: Neutral {
    fn initial_amount(&self) -> isize;
}

impl AsNeutral for ResourceData {}

neutral_delegate!(ResourceData);

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

pub struct Mineral {
    initial_amount: isize,
    neutral: NeutralData,
}

impl AsNeutral for Mineral {
    // If this Neutral is a Mineral, returns a typed pointer to this Mineral.
    // Otherwise, returns nullptr.
    fn is_mineral(&self) -> Option<&Mineral> {
        Some(self)
    }
}

impl Resource for Mineral {
    fn initial_amount(&self) -> isize {
        self.initial_amount
    }
}

impl Mineral {
    pub fn new(unit: &Unit, map: &Map) -> Self {
        unimplemented!()
    }
}

neutral_delegate!(Mineral);

///////////////////////////////////////////////////////////////////////////////////////////////
///                                                                                          //
/// class Geyser
///                                                                                          //
///////////////////////////////////////////////////////////////////////////////////////////////
///
/// Geysers Correspond to the units in BWAPI::getStaticNeutralUnits() for which getType() == Resource_Vespene_Geyser
pub struct Geyser {
    initial_amount: isize,
    neutral: NeutralData,
}

impl AsNeutral for Geyser {
    /// If this Neutral is a Geyser, returns a typed pointer to this Geyser.
    /// Otherwise, returns nullptr.
    fn is_geyser(&self) -> Option<&Geyser> {
        Some(self)
    }
}

impl Resource for Geyser {
    fn initial_amount(&self) -> isize {
        self.initial_amount
    }
}

impl Geyser {
    pub fn new(unit: &Unit, map: &Map) -> Self {
        unimplemented!()
    }
}

neutral_delegate!(Geyser);

///////////////////////////////////////////////////////////////////////////////////////////////
///                                                                                          //
/// class StaticBuilding
///                                                                                          //
///////////////////////////////////////////////////////////////////////////////////////////////
///
/// StaticBuildings Correspond to the units in BWAPI::getStaticNeutralUnits() for which getType().isSpecialBuilding
/// StaticBuilding also wrappers some special units like Special_Pit_Door.
pub struct StaticBuilding {
    neutral: NeutralData,
}

impl AsNeutral for StaticBuilding {
    /// If this Neutral is a StaticBuilding, returns a typed pointer to this StaticBuilding.
    /// Otherwise, returns nullptr.
    fn is_static_building(&self) -> Option<&StaticBuilding> {
        Some(self)
    }
}

impl StaticBuilding {
    pub fn new(unit: &Unit, map: &Map) -> Self {
        unimplemented!()
    }
}

neutral_delegate!(StaticBuilding);
