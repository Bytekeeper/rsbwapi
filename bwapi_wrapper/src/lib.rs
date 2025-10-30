#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use crate::prelude::{UnitType, UpgradeType};

pub mod command;
pub mod tech_type;
pub mod unit_type;
pub mod upgrade_type;
pub mod weapon_type;

pub mod position;
pub mod prelude;

#[allow(clippy::all)]
mod bindings {
    use num_derive::FromPrimitive;
    // include!("bindings.rs");
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
pub use bindings::*;

pub trait TypeFrom {
    fn new(i: i32) -> Self;
}

impl<T: num_traits::FromPrimitive> TypeFrom for T {
    fn new(i: i32) -> Self {
        Self::from_i32(i).unwrap()
    }
}

impl UnitType {
    pub fn is_successor_of(&self, type_: UnitType) -> bool {
        if type_ == *self {
            return true;
        }
        match type_ {
            UnitType::Zerg_Hatchery => *self == UnitType::Zerg_Lair || *self == UnitType::Zerg_Hive,
            UnitType::Zerg_Lair => *self == UnitType::Zerg_Hive,
            UnitType::Zerg_Spire => *self == UnitType::Zerg_Greater_Spire,
            _ => false,
        }
    }
}

impl Default for UnitType {
    fn default() -> Self {
        Self::None
    }
}

// DEFAULTS
const DEFAULT_ORE_COST_BASE: [i32; UpgradeType::MAX as usize] =
    // same as default gas cost base
    [
        100, 100, 150, 150, 150, 100, 150, 100, 100, 100, 100, 100, 100, 100, 100, 200, 150, 100,
        200, 150, 100, 150, 200, 150, 200, 150, 150, 100, 200, 150, 150, 150, 150, 150, 150, 200,
        200, 200, 150, 150, 150, 100, 200, 100, 150, 0, 0, 100, 100, 150, 150, 150, 150, 200, 100,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

const DEFAULT_TIME_COST_BASE: [i32; UpgradeType::MAX as usize] = [
    4000, 4000, 4000, 4000, 4000, 4000, 4000, 4000, 4000, 4000, 4000, 4000, 4000, 4000, 4000, 4000,
    1500, 1500, 0, 2500, 2500, 2500, 2500, 2500, 2400, 2000, 2000, 1500, 1500, 1500, 1500, 2500,
    2500, 2500, 2000, 2500, 2500, 2500, 2000, 2000, 2500, 2500, 2500, 1500, 2500, 0, 0, 2500, 2500,
    2500, 2500, 2500, 2000, 2000, 2000, 0, 0, 0, 0, 0, 0, 0, 0,
];

mod upgrade_internals {
    use crate::BWAPI_UnitTypes_Enum_Enum::*;
    use crate::prelude::{UnitType, UpgradeType};

    pub(crate) const REQUIREMENTS: [[UnitType; UpgradeType::MAX as usize]; 3] = [
        // Level 1
        [
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Zerg_Hive,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Terran_Armory,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        // Level 2
        [
            Terran_Science_Facility,
            Terran_Science_Facility,
            Terran_Science_Facility,
            Zerg_Lair,
            Zerg_Lair,
            Protoss_Templar_Archives,
            Protoss_Fleet_Beacon,
            Terran_Science_Facility,
            Terran_Science_Facility,
            Terran_Science_Facility,
            Zerg_Lair,
            Zerg_Lair,
            Zerg_Lair,
            Protoss_Templar_Archives,
            Protoss_Fleet_Beacon,
            Protoss_Cybernetics_Core,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        // Level 3
        [
            Terran_Science_Facility,
            Terran_Science_Facility,
            Terran_Science_Facility,
            Zerg_Hive,
            Zerg_Hive,
            Protoss_Templar_Archives,
            Protoss_Fleet_Beacon,
            Terran_Science_Facility,
            Terran_Science_Facility,
            Terran_Science_Facility,
            Zerg_Hive,
            Zerg_Hive,
            Zerg_Hive,
            Protoss_Templar_Archives,
            Protoss_Fleet_Beacon,
            Protoss_Cybernetics_Core,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
    ];
}

impl UpgradeType {
    pub fn mineral_price(&self, level: i32) -> i32 {
        DEFAULT_ORE_COST_BASE[*self as usize] + 0.max(level - 1) * self.mineral_price_factor()
    }

    pub fn gas_price(&self, level: i32) -> i32 {
        self.mineral_price(level)
    }

    pub fn upgrade_time(&self, level: i32) -> i32 {
        DEFAULT_TIME_COST_BASE[*self as usize] + 0.max(level - 1) * self.upgrade_time_factor()
    }

    pub fn whats_required(&self, level: i32) -> UnitType {
        if (1..=3).contains(&level) {
            upgrade_internals::REQUIREMENTS[level as usize - 1][*self as usize]
        } else {
            UnitType::None
        }
    }
}
