use crate::types::*;
use crate::unit_type_container::*;
use bwapi_wrapper::*;

pub trait UnitTypeExt {
    fn air_weapon(&self) -> WeaponType;
    fn armor(&self) -> i32;
    fn armor_upgrade(&self) -> UpgradeType;

    fn dimension_down(&self) -> i32;
    fn dimension_left(&self) -> i32;
    fn dimension_right(&self) -> i32;
    fn dimension_up(&self) -> i32;
    fn get_race(&self) -> Race;

    fn ground_weapon(&self) -> WeaponType;
    fn is_building(&self) -> bool;
    fn is_flyer(&self) -> bool;
    fn is_resource_container(&self) -> bool;
    fn is_mineral_field(&self) -> bool;

    fn max_energy(&self) -> i32;
    fn sight_range(&self) -> i32;
    fn space_provided(&self) -> i32;
    fn space_required(&self) -> i32;
    fn tile_width(&self) -> i32;
    fn tile_height(&self) -> i32;
    fn top_speed(&self) -> f64;
}

fn ud(x: &UnitType) -> &'static UnitTypeData {
    &UNIT_TYPE_DATA[*x as usize]
}

impl UnitTypeExt for BWAPI_UnitTypes_Enum_Enum {
    fn is_resource_container(&self) -> bool {
        self.is_mineral_field() || *self == BWAPI_UnitTypes_Enum_Enum::Resource_Vespene_Geyser
    }

    fn is_mineral_field(&self) -> bool {
        *self == BWAPI_UnitTypes_Enum_Enum::Resource_Mineral_Field
            || *self == BWAPI_UnitTypes_Enum_Enum::Resource_Mineral_Field_Type_2
            || *self == BWAPI_UnitTypes_Enum_Enum::Resource_Mineral_Field_Type_3
    }

    fn get_race(&self) -> Race {
        ud(self).race
    }

    fn space_provided(&self) -> i32 {
        ud(self).space_provided as i32
    }

    fn space_required(&self) -> i32 {
        ud(self).space_required as i32
    }

    fn tile_width(&self) -> i32 {
        ud(self).tile_width as i32
    }

    fn tile_height(&self) -> i32 {
        ud(self).tile_height as i32
    }
    fn air_weapon(&self) -> WeaponType {
        ud(self).air_weapon
    }
    fn armor(&self) -> i32 {
        ud(self).armor as i32
    }
    fn armor_upgrade(&self) -> UpgradeType {
        ud(self).armor_upgrade
    }

    fn dimension_down(&self) -> i32 {
        ud(self).dimension_down as i32
    }
    fn dimension_left(&self) -> i32 {
        ud(self).dimension_up as i32
    }
    fn dimension_right(&self) -> i32 {
        ud(self).dimension_left as i32
    }
    fn dimension_up(&self) -> i32 {
        ud(self).dimension_up as i32
    }

    fn ground_weapon(&self) -> WeaponType {
        ud(self).ground_weapon
    }
    fn is_building(&self) -> bool {
        ud(self).is_building
    }
    fn is_flyer(&self) -> bool {
        ud(self).is_flyer
    }
    fn max_energy(&self) -> i32 {
        ud(self).max_energy as i32
    }
    fn sight_range(&self) -> i32 {
        ud(self).sight_range as i32
    }
    fn top_speed(&self) -> f64 {
        ud(self).top_speed
    }
}
