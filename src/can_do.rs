//!
//! Helper traits and impls for the various can_xyz methods.
//! BWAPI uses multiple overloaded methods, which is emulated using traits. Therefore, traits in here are
//! usually implemented for `bool` and for one or more tuples. The `bool` in this case is the parameter `checkCommandibility`.
//! Be warned, all the checking API is not stable, as it is mostly a rough conversion from C++

use crate::*;

#[derive(Clone, Copy, Debug)]
pub struct CanIssueCommandArg {
    pub c: UnitCommand,
    pub check_can_use_tech_position_on_positions: bool,
    pub check_can_use_tech_unit_on_units: bool,
    pub check_can_build_unit_type: bool,
    pub check_can_target_unit: bool,
    pub check_can_issue_command_type: bool,
    pub check_commandability: bool,
}

impl CanIssueCommandArg {
    fn new(c: UnitCommand) -> Self {
        Self {
            c,
            check_can_use_tech_position_on_positions: true,
            check_can_use_tech_unit_on_units: true,
            check_can_build_unit_type: true,
            check_can_target_unit: true,
            check_can_issue_command_type: true,
            check_commandability: true,
        }
    }
}

impl From<UnitCommand> for CanIssueCommandArg {
    fn from(cmd: UnitCommand) -> Self {
        Self::new(cmd)
    }
}

pub trait CanAttackUnit {
    fn can_attack_unit(&self, unit: &Unit) -> BwResult<bool>;
}

pub trait CanAttack {
    fn can_attack(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanAttack for bool {
    fn can_attack(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.can_attack_move(false).unwrap_or(false) && !false.can_attack_unit(unit)? {
            return Ok(false);
        }
        Ok(true)
    }
}
impl CanAttackUnit for bool {
    fn can_attack_unit(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.get_type().is_building() && !unit.is_interruptible() {
            return Err(Error::Unit_Busy);
        }

        if unit.get_type().ground_weapon() == WeaponType::None
            && unit.get_type().air_weapon() == WeaponType::None
        {
            if unit.get_type() == UnitType::Protoss_Carrier
                || unit.get_type() == UnitType::Hero_Gantrithor
            {
                if unit.get_interceptor_count() <= 0 {
                    return Err(Error::Unable_To_Hit);
                }
            } else if unit.get_type() == UnitType::Protoss_Reaver
                || unit.get_type() == UnitType::Hero_Warbringer
            {
                if unit.get_scarab_count() <= 0 {
                    return Err(Error::Unable_To_Hit);
                }
            } else {
                return Err(Error::Unable_To_Hit);
            }
        }
        if unit.get_type() == UnitType::Zerg_Lurker {
            if !unit.is_burrowed() {
                return Err(Error::Unable_To_Hit);
            }
        } else if unit.is_burrowed() {
            return Err(Error::Unable_To_Hit);
        } else if !unit.is_completed() {
            return Err(Error::Incompatible_State);
        } else if unit.get_order() == Order::ConstructingBuilding {
            return Err(Error::Unit_Busy);
        }
        Ok(true)
    }
}

impl CanAttack for (Position, bool, bool, bool) {
    fn can_attack(&self, unit: &Unit) -> BwResult<bool> {
        let (_, _, _, check_commandability) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.can_attack_move(false)? {
            return Ok(false);
        }
        Ok(true)
    }
}

impl CanAttack for (Option<&Unit<'_>>, bool, bool, bool) {
    fn can_attack(&self, unit: &Unit) -> BwResult<bool> {
        let (target, check_can_target_unit, check_can_issue_command_type, check_commandability) =
            *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }
        let target = target.ok_or(Error::Invalid_Parameter)?;
        if !(
            target,
            check_can_target_unit,
            check_can_issue_command_type,
            false,
        )
            .can_attack_unit(unit)?
        {
            return Ok(false);
        }
        Ok(true)
    }
}
impl CanAttackUnit for (&Unit<'_>, bool, bool, bool) {
    fn can_attack_unit(&self, unit: &Unit) -> BwResult<bool> {
        let (
            target_unit,
            check_can_target_unit,
            check_can_issue_command_type,
            check_commandability,
        ) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_attack_unit(false)? {
            return Ok(false);
        }
        if check_can_target_unit && !unit.game.can_target_unit(target_unit)? {
            return Ok(false);
        }

        if target_unit.is_invincible() {
            return Err(Error::Unable_To_Hit);
        }
        let type_ = unit.get_type();
        let target_in_air = target_unit.is_flying();
        let weapon = if target_in_air {
            type_.air_weapon()
        } else {
            type_.ground_weapon()
        };
        if weapon == WeaponType::None {
            match type_ {
                UnitType::Protoss_Carrier | UnitType::Hero_Gantrithor => (),
                UnitType::Protoss_Reaver | UnitType::Hero_Warbringer => {
                    if target_in_air {
                        return Err(Error::Unable_To_Hit);
                    }
                }
                _ => return Err(Error::Unable_To_Hit),
            }
        }

        if !unit.get_type().can_move() && !unit.is_in_weapon_range(target_unit) {
            return Err(Error::Out_Of_Range);
        }

        if unit.get_type() == UnitType::Zerg_Lurker && !unit.is_in_weapon_range(target_unit) {
            return Err(Error::Out_Of_Range);
        }

        if unit == target_unit {
            return Err(Error::Invalid_Parameter);
        }

        Ok(true)
    }
}

/// Argument tuple to check if some unit can build some building
pub trait CanBuild {
    fn can_build(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanBuild for bool {
    fn can_build(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.get_type().is_building() && !unit.is_interruptible() {
            return Err(Error::Unit_Busy);
        }
        if unit.is_constructing()
            || !unit.is_completed()
            || (unit.get_type().is_building() && !unit.is_idle())
        {
            return Err(Error::Unit_Busy);
        }
        if unit.is_hallucination() {
            return Err(Error::Incompatible_UnitType);
        }
        Ok(true)
    }
}

impl CanBuild for (UnitType, bool, bool) {
    fn can_build(&self, unit: &Unit) -> BwResult<bool> {
        let (u_type, check_can_issue_command_type, check_commandability) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && unit.can_build(false)? {
            return Ok(false);
        }

        if !unit.game.can_make(unit, u_type)? {
            return Ok(false);
        }

        if !u_type.is_building() {
            return Err(Error::Incompatible_UnitType);
        }
        if unit.get_addon().is_some() {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }
}

impl CanBuild for (UnitType, TilePosition, bool, bool, bool) {
    fn can_build(&self, unit: &Unit) -> BwResult<bool> {
        let (
            u_type,
            tile_pos,
            check_target_unit_type,
            check_can_issue_command_type,
            check_commandability,
        ) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_build(false)? {
            return Ok(false);
        }

        if check_target_unit_type && !unit.can_build((u_type, false, false))? {
            return Ok(false);
        }
        if !tile_pos.is_valid() {
            return Err(Error::Invalid_Tile_Position);
        }
        if !unit.game.can_build_here(unit, tile_pos, u_type, true)? {
            return Ok(false);
        }
        Ok(true)
    }
}

pub trait CanBuildAddon {
    fn can_build_addon(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanBuildAddon for bool {
    fn can_build_addon(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }
        if unit.is_constructing()
            || !unit.is_completed()
            || unit.is_lifted()
            || (unit.get_type().is_building() && !unit.is_idle())
        {
            return Err(Error::Unit_Busy);
        }
        if unit.get_addon().is_some() {
            return Err(Error::Incompatible_State);
        }
        if !unit.get_type().can_build_addon() {
            return Err(Error::Incompatible_UnitType);
        }
        Ok(true)
    }
}

impl CanBuildAddon for (UnitType, bool, bool) {
    fn can_build_addon(&self, unit: &Unit) -> BwResult<bool> {
        let (u_type, check_can_issue_command_type, check_commandability) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_build_addon(false)? {
            return Ok(false);
        }

        if !unit.game.can_make(unit, u_type)? {
            return Ok(false);
        }

        if !u_type.is_addon() {
            return Err(Error::Incompatible_UnitType);
        }

        if !unit
            .game
            .can_build_here(unit, unit.get_tile_position(), u_type, true)?
        {
            return Ok(false);
        }
        Ok(true)
    }
}

pub trait CanTrain {
    fn can_train(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanTrain for bool {
    fn can_train(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }
        if unit.get_type().produces_larva() {
            if !unit.is_constructing() && unit.is_completed() {
                return Ok(true);
            }
            for larva in unit.get_larva() {
                if !larva.is_constructing()
                    && larva.is_completed()
                    && larva.can_command().unwrap_or(false)
                {
                    return Ok(true);
                }
            }
            return Err(Error::Unit_Busy);
        }
        if unit.is_constructing() || !unit.is_completed() || unit.is_lifted() {
            return Err(Error::Unit_Busy);
        }
        if !unit.get_type().can_produce()
            && unit.get_type() != UnitType::Terran_Nuclear_Missile
            && unit.get_type() != UnitType::Zerg_Hydralisk
            && unit.get_type() != UnitType::Zerg_Mutalisk
            && unit.get_type() != UnitType::Zerg_Creep_Colony
            && unit.get_type() != UnitType::Zerg_Spire
            && unit.get_type() != UnitType::Zerg_Larva
        {
            return Err(Error::Incompatible_UnitType);
        }
        if unit.is_hallucination() {
            return Err(Error::Incompatible_UnitType);
        }
        Ok(true)
    }
}

impl CanTrain for (UnitType, bool, bool) {
    fn can_train(&self, unit: &Unit) -> BwResult<bool> {
        let (u_type, check_can_issue_command_type, check_commandability) = *self;

        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_train(false)? {
            return Ok(false);
        }

        let unit = if unit.get_type().produces_larva() {
            if u_type.what_builds().0 == UnitType::Zerg_Larva {
                unit.get_larva()
                    .iter()
                    .find(|u| u.can_train(true).unwrap_or(false))
                    .cloned()
                    .unwrap_or(*unit)
            } else if unit.is_constructing() || !unit.is_completed() {
                return Err(Error::Unit_Busy);
            } else {
                *unit
            }
        } else {
            *unit
        };

        if !unit.game.can_make(&unit, u_type)? {
            return Ok(false);
        }

        if u_type.is_addon() || (u_type.is_building() && !unit.get_type().is_building()) {
            return Err(Error::Incompatible_UnitType);
        }
        if u_type == UnitType::Zerg_Larva
            || u_type == UnitType::Zerg_Egg
            || u_type == UnitType::Zerg_Cocoon
        {
            return Err(Error::Incompatible_UnitType);
        }

        Ok(true)
    }
}

pub trait CanMorph {
    fn can_morph(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanMorph for bool {
    fn can_morph(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }

        if unit.get_type().produces_larva() {
            if !unit.is_constructing()
                && unit.is_completed()
                && !(unit.get_type().is_building() || unit.is_idle())
            {
                return Ok(true);
            }
            let larva_available = unit.get_larva().iter().any(|u| {
                !u.is_constructing() && u.is_completed() && u.can_command().unwrap_or(false)
            });
            if larva_available {
                return Ok(true);
            }
            return Err(Error::Unit_Busy);
        }

        if unit.is_constructing()
            || !unit.is_completed()
            || (unit.get_type().is_building() && !unit.is_idle())
        {
            return Err(Error::Unit_Busy);
        }
        if unit.get_type() != UnitType::Zerg_Hydralisk
            && unit.get_type() != UnitType::Zerg_Mutalisk
            && unit.get_type() != UnitType::Zerg_Creep_Colony
            && unit.get_type() != UnitType::Zerg_Spire
            && unit.get_type() != UnitType::Zerg_Hatchery
            && unit.get_type() != UnitType::Zerg_Lair
            && unit.get_type() != UnitType::Zerg_Hive
            && unit.get_type() != UnitType::Zerg_Larva
        {
            return Err(Error::Incompatible_UnitType);
        }
        if unit.is_hallucination() {
            return Err(Error::Incompatible_UnitType);
        }
        Ok(true)
    }
}

impl CanMorph for (UnitType, bool, bool) {
    fn can_morph(&self, unit: &Unit) -> BwResult<bool> {
        let (u_type, check_can_issue_command_type, check_commandability) = *self;

        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_morph(false)? {
            return Ok(false);
        }

        let unit = if unit.get_type().produces_larva() {
            if u_type.what_builds().0 == UnitType::Zerg_Larva {
                unit.get_larva()
                    .iter()
                    .find(|u| u.can_morph(true).unwrap_or(false))
                    .cloned()
                    .ok_or(Error::Unit_Does_Not_Exist)?
            } else if unit.is_constructing()
                || !unit.is_completed()
                || (unit.get_type().is_building() && !unit.is_idle())
            {
                return Err(Error::Unit_Busy);
            } else {
                *unit
            }
        } else {
            *unit
        };

        if !unit.game.can_make(&unit, u_type)? {
            return Ok(false);
        }

        if u_type == UnitType::Zerg_Larva
            || u_type == UnitType::Zerg_Egg
            || u_type == UnitType::Zerg_Cocoon
        {
            return Err(Error::Incompatible_UnitType);
        }

        Ok(true)
    }
}

pub trait CanFollow {
    fn can_follow(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanFollow for bool {
    fn can_follow(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.can_move(false)? {
            Ok(false)
        } else {
            Ok(true)
        }
    }
}

impl CanFollow for (&Unit<'_>, bool, bool, bool) {
    fn can_follow(&self, unit: &Unit) -> BwResult<bool> {
        let (
            target_unit,
            check_can_target_unit,
            check_can_issue_command_type,
            check_commandability,
        ) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_follow(false)? {
            return Ok(false);
        }
        if check_can_target_unit && !unit.game.can_target_unit((unit, target_unit, false))? {
            return Ok(false);
        }

        if unit == target_unit {
            return Err(Error::Invalid_Parameter);
        }
        Ok(true)
    }
}

pub trait CanSetRallyUnit {
    fn can_set_rally_unit(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanSetRallyUnit for bool {
    fn can_set_rally_unit(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }
        if !unit.get_type().can_produce() || !unit.get_type().is_building() {
            return Err(Error::Incompatible_UnitType);
        }
        if unit.is_lifted() {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }
}

impl CanSetRallyUnit for (&Unit<'_>, bool, bool, bool) {
    fn can_set_rally_unit(&self, unit: &Unit) -> BwResult<bool> {
        let (
            target_unit,
            check_can_target_unit,
            check_can_issue_command_type,
            check_commandability,
        ) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }
        if check_can_issue_command_type && !unit.can_set_rally_unit(false)? {
            return Ok(false);
        }
        if check_can_target_unit && !unit.game.can_target_unit((unit, target_unit, false))? {
            return Ok(false);
        }
        Ok(true)
    }
}

pub trait CanGather {
    fn can_gather(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanGather for bool {
    fn can_gather(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.get_type().is_building() && !unit.is_interruptible() {
            return Err(Error::Unit_Busy);
        }
        if !unit.get_type().is_worker() {
            return Err(Error::Incompatible_UnitType);
        }
        if unit.get_power_up().is_some() {
            return Err(Error::Unit_Busy);
        }
        if unit.is_hallucination() {
            return Err(Error::Incompatible_UnitType);
        }
        if unit.is_burrowed() {
            return Err(Error::Incompatible_State);
        }
        if !unit.is_completed() {
            return Err(Error::Incompatible_State);
        }
        if unit.get_order() == Order::ConstructingBuilding {
            return Err(Error::Unit_Busy);
        }
        Ok(true)
    }
}

impl CanGather for (&Unit<'_>, bool, bool, bool) {
    fn can_gather(&self, unit: &Unit) -> BwResult<bool> {
        let (
            target_unit,
            check_can_target_unit,
            check_can_issue_command_type,
            check_commandability,
        ) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }
        if check_can_issue_command_type && !unit.can_gather(false)? {
            return Ok(false);
        }
        if check_can_target_unit && !unit.game.can_target_unit((unit, target_unit, false))? {
            return Ok(false);
        }
        let u_type = target_unit.get_type();
        if !u_type.is_resource_container() || u_type == UnitType::Resource_Vespene_Geyser {
            return Err(Error::Incompatible_UnitType);
        }
        if !target_unit.is_completed() {
            return Err(Error::Unit_Busy);
        }
        if !unit.has_path(target_unit.get_position()) {
            return Err(Error::Unreachable_Location);
        }
        if u_type.is_refinery() && Some(target_unit.get_player()) != unit.game.self_() {
            return Err(Error::Unit_Not_Owned);
        }
        Ok(true)
    }
}

pub trait CanRepair {
    fn can_repair(&self, unit: &Unit) -> BwResult<bool>;
}
impl CanRepair for bool {
    fn can_repair(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.is_interruptible() {
            return Err(Error::Unit_Busy);
        }
        if unit.get_type() != UnitType::Terran_SCV {
            return Err(Error::Incompatible_UnitType);
        }
        if !unit.is_completed() {
            return Err(Error::Incompatible_State);
        }
        if unit.is_hallucination() {
            return Err(Error::Incompatible_UnitType);
        }
        if unit.get_order() == Order::ConstructingBuilding {
            return Err(Error::Unit_Busy);
        }
        Ok(true)
    }
}

impl CanRepair for (&Unit<'_>, bool, bool, bool) {
    fn can_repair(&self, unit: &Unit) -> BwResult<bool> {
        let (
            target_unit,
            check_can_target_unit,
            check_can_issue_command_type,
            check_commandability,
        ) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_repair(false)? {
            return Ok(false);
        }
        if check_can_target_unit && !unit.game.can_target_unit((unit, target_unit, false))? {
            return Ok(false);
        }

        let targ_type = target_unit.get_type();
        if targ_type.get_race() != Race::Terran || !targ_type.is_mechanical() {
            return Err(Error::Incompatible_UnitType);
        }
        if target_unit.get_hit_points() == targ_type.max_hit_points() {
            return Err(Error::Incompatible_State);
        }
        if !target_unit.is_completed() {
            return Err(Error::Incompatible_State);
        }
        if target_unit == unit {
            return Err(Error::Invalid_Parameter);
        }
        Ok(true)
    }
}

pub trait CanLand {
    fn can_land(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanLand for bool {
    fn can_land(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.get_type().is_flying_building() {
            return Err(Error::Incompatible_UnitType);
        }
        if !unit.is_lifted() {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }
}

impl CanLand for (TilePosition, bool, bool) {
    fn can_land(&self, unit: &Unit) -> BwResult<bool> {
        let (target, check_can_issue_command_type, check_commandability) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_land(false)? {
            return Ok(false);
        }
        if !unit
            .game
            .can_build_here(None, target, unit.get_type(), true)?
        {
            return Ok(false);
        }
        Ok(true)
    }
}

pub trait CanLoad {
    fn can_load(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanLoad for bool {
    fn can_load(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.get_type().is_building() && !unit.is_interruptible() {
            return Err(Error::Unit_Busy);
        }
        if !unit.is_completed() {
            return Err(Error::Unit_Busy);
        }
        if unit.get_type() == UnitType::Zerg_Overlord
            && unit
                .game
                .self_()
                .expect("Player self to exist")
                .get_upgrade_level(UpgradeType::Ventral_Sacs)
                == 0
        {
            return Err(Error::Insufficient_Tech);
        }
        if unit.is_burrowed() {
            return Err(Error::Incompatible_State);
        }
        if unit.get_order() == Order::ConstructingBuilding {
            return Err(Error::Unit_Busy);
        }
        if unit.get_type() == UnitType::Zerg_Larva {
            return Err(Error::Incompatible_UnitType);
        }
        Ok(true)
    }
}

impl CanLoad for (&Unit<'_>, bool, bool, bool) {
    fn can_load(&self, unit: &Unit) -> BwResult<bool> {
        let (
            target_unit,
            check_can_target_unit,
            check_can_issue_command_type,
            check_commandability,
        ) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_load(false)? {
            return Ok(false);
        }

        if check_can_target_unit && !unit.game.can_target_unit((unit, target_unit, false))? {
            return Ok(false);
        }

        if Some(target_unit.get_player()) != unit.game.self_() {
            return Err(Error::Unit_Not_Owned);
        }

        if target_unit.is_loaded() || !target_unit.is_completed() {
            return Err(Error::Unit_Busy);
        }

        if target_unit.get_type() == UnitType::Zerg_Overlord
            && unit
                .game
                .self_()
                .expect("Self to exist")
                .get_upgrade_level(UpgradeType::Ventral_Sacs)
                == 0
        {
            return Err(Error::Insufficient_Tech);
        }

        let this_unit_space_provided = unit.get_type().space_provided();
        let target_space_provided = target_unit.get_type().space_provided();
        if this_unit_space_provided <= 0 && target_space_provided <= 0 {
            return Err(Error::Incompatible_UnitType);
        }

        let unit_to_be_loaded = if this_unit_space_provided > 0 {
            target_unit
        } else {
            unit
        };
        if !unit_to_be_loaded.get_type().can_move()
            || unit_to_be_loaded.get_type().is_flyer()
            || unit_to_be_loaded.get_type().space_required() > 8
        {
            return Err(Error::Incompatible_UnitType);
        }

        if !unit_to_be_loaded.is_completed() {
            return Err(Error::Incompatible_State);
        }

        if unit_to_be_loaded.is_burrowed() {
            return Err(Error::Incompatible_State);
        }

        let unit_that_loads = if this_unit_space_provided > 0 {
            unit
        } else {
            target_unit
        };
        if unit_that_loads.is_hallucination() {
            return Err(Error::Incompatible_UnitType);
        }

        if unit_that_loads.get_type() == UnitType::Terran_Bunker {
            if !unit_to_be_loaded.get_type().is_organic()
                || unit_to_be_loaded.get_type().get_race() != Race::Terran
            {
                return Err(Error::Incompatible_UnitType);
            }
            if !unit_to_be_loaded.has_path(unit_that_loads) {
                return Err(Error::Unreachable_Location);
            }
        }

        let mut free_space = if this_unit_space_provided > 0 {
            this_unit_space_provided
        } else {
            target_space_provided
        };
        free_space -= unit_that_loads
            .get_loaded_units()
            .iter()
            .map(|u| u.get_type().space_required())
            .filter(|&n| n > 0 && n < 8)
            .sum::<i32>();
        if unit_to_be_loaded.get_type().space_required() > free_space {
            return Err(Error::Insufficient_Space);
        }

        Ok(true)
    }
}

pub trait CanUnload {
    fn can_unload(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanUnload for bool {
    fn can_unload(&self, unit: &Unit) -> BwResult<bool> {
        unit.can_unload_at_position(unit.get_position(), true, *self)
    }
}

impl CanUnload for (&Unit<'_>, bool, bool, bool, bool) {
    fn can_unload(&self, unit: &Unit) -> BwResult<bool> {
        let (
            target_unit,
            check_can_target_unit,
            check_position,
            check_can_issue_command_type,
            check_commandability,
        ) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_unload_with_or_without_target(false)? {
            return Ok(false);
        }

        if check_position && !unit.can_unload_at_position(unit.get_position(), false, false)? {
            return Ok(false);
        }

        if check_can_target_unit && !unit.game.can_target_unit((unit, target_unit, false))? {
            return Ok(false);
        }

        if !target_unit.is_loaded() {
            return Err(Error::Incompatible_State);
        }

        if target_unit.get_transport() != Some(*unit) {
            return Err(Error::Invalid_Parameter);
        }
        Ok(true)
    }
}

pub trait CanUnloadAllPosition {
    fn can_unload_all_position(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanUnloadAllPosition for bool {
    fn can_unload_all_position(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.can_unload_with_or_without_target(false)? {
            return Ok(false);
        }

        if unit.get_type() == UnitType::Terran_Bunker {
            return Err(Error::Incompatible_UnitType);
        }
        Ok(true)
    }
}

impl CanUnloadAllPosition for (Position, bool, bool) {
    fn can_unload_all_position(&self, unit: &Unit) -> BwResult<bool> {
        let (targ_drop_pos, check_can_issue_command_type, check_commandability) = *self;

        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && unit.can_unload_all_position(false)? {
            return Ok(false);
        }

        if !unit.can_unload_at_position(targ_drop_pos, false, false)? {
            return Ok(false);
        }
        Ok(true)
    }
}

pub trait CanRightClickUnit {
    fn can_right_click_unit(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanRightClickUnit for bool {
    fn can_right_click_unit(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.get_type().is_building() && !unit.is_interruptible() {
            return Err(Error::Unit_Busy);
        }
        if !unit.can_follow(false).unwrap_or(false)
            && !unit.can_attack_unit(false).unwrap_or(false)
            && !unit.can_load(false).unwrap_or(false)
            && !unit.can_set_rally_unit(false).unwrap_or(false)
        {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }
}

impl CanRightClickUnit for (&Unit<'_>, bool, bool, bool) {
    fn can_right_click_unit(&self, unit: &Unit) -> BwResult<bool> {
        let (
            target_unit,
            check_can_target_unit,
            check_can_issue_command_type,
            check_commandability,
        ) = *self;

        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_right_click_unit(false)? {
            return Ok(false);
        }

        if check_can_target_unit && !unit.game.can_target_unit((unit, target_unit, false))? {
            return Ok(false);
        }

        if !target_unit.get_player().is_neutral()
            && unit.get_player().is_enemy(&target_unit.get_player())
            && !unit.can_attack_unit((target_unit, false, true, false))?
        {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }
}

pub trait CanCancelTrainSlot {
    fn can_cancel_train_slot(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanCancelTrainSlot for bool {
    fn can_cancel_train_slot(&self, unit: &Unit) -> BwResult<bool> {
        unit.can_cancel_train(*self)
    }
}

impl CanCancelTrainSlot for (i32, bool, bool) {
    fn can_cancel_train_slot(&self, unit: &Unit) -> BwResult<bool> {
        let (slot, check_can_issue_command_type, check_commandability) = *self;

        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_cancel_train_slot(false)? {
            return Ok(false);
        }

        if !unit.is_training() || (unit.get_training_queue().len() as i32 <= slot && slot >= 0) {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }
}

pub trait CanUseTechWithOrWithoutTarget {
    fn can_use_tech_with_or_without_target(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanUseTechWithOrWithoutTarget for bool {
    fn can_use_tech_with_or_without_target(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.get_type().is_building() && !unit.is_interruptible() {
            return Err(Error::Unit_Busy);
        }
        if !unit.is_completed() {
            return Err(Error::Incompatible_State);
        }
        if unit.is_hallucination() {
            return Err(Error::Incompatible_UnitType);
        }
        Ok(true)
    }
}

impl CanUseTechWithOrWithoutTarget for (TechType, bool, bool) {
    fn can_use_tech_with_or_without_target(&self, unit: &Unit) -> BwResult<bool> {
        let (tech, check_can_issue_command_type, check_commandability) = *self;

        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_use_tech_with_or_without_target(false)? {
            return Ok(false);
        }

        if !unit.get_type().is_hero()
            && !unit
                .game
                .self_()
                .expect("Self to exist")
                .has_researched(tech)
            && unit.get_type() == UnitType::Zerg_Lurker
        {
            return Err(Error::Insufficient_Tech);
        }

        if unit.get_energy() < tech.energy_cost() {
            return Err(Error::Insufficient_Energy);
        }

        if tech != TechType::Burrowing && !tech.what_uses().contains(&unit.get_type()) {
            return Err(Error::Incompatible_UnitType);
        }

        match tech {
            TechType::Spider_Mines => {
                if unit.get_spider_mine_count() <= 0 {
                    return Err(Error::Insufficient_Ammo);
                }
            }
            TechType::Tank_Siege_Mode => {
                if unit.is_sieged() {
                    return Err(Error::Incompatible_State);
                }
                if unit.get_order() == Order::Sieging || unit.get_order() == Order::Unsieging {
                    return Err(Error::Unit_Busy);
                }
            }
            TechType::Cloaking_Field | TechType::Personnel_Cloaking => {
                if unit.get_secondary_order() == Order::Cloak {
                    return Err(Error::Incompatible_State);
                }
            }
            TechType::Burrowing => {
                if unit.get_type().is_burrowable() {
                    return Err(Error::Incompatible_UnitType);
                }
                if unit.is_burrowed()
                    || unit.get_order() == Order::Burrowing
                    || unit.get_order() == Order::Unburrowing
                {
                    return Err(Error::Incompatible_State);
                }
            }
            _ => (),
        }
        Ok(true)
    }
}

pub trait CanUseTechUnit {
    fn can_use_tech_unit(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanUseTechUnit for (TechType, bool, bool) {
    fn can_use_tech_unit(&self, unit: &Unit) -> BwResult<bool> {
        let (tech, check_can_issue_command_type, check_commandability) = *self;

        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_use_tech_with_or_without_target(false)? {
            return Ok(false);
        }

        if !tech.targets_unit() {
            return Err(Error::Incompatible_TechType);
        }
        Ok(true)
    }
}

impl CanUseTechUnit for (TechType, &Unit<'_>, bool, bool, bool, bool) {
    fn can_use_tech_unit(&self, unit: &Unit) -> BwResult<bool> {
        let (
            tech,
            target_unit,
            check_can_target_unit,
            check_targets_units,
            check_can_issue_command_type,
            check_commandability,
        ) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }
        if check_can_issue_command_type && !unit.can_use_tech_with_or_without_target(false)? {
            return Ok(false);
        }
        if check_targets_units && !unit.can_use_tech_unit((tech, false, false))? {
            return Ok(false);
        }
        if check_can_target_unit && !unit.game.can_target_unit((unit, target_unit, false))? {
            return Ok(false);
        }
        let target_type = target_unit.get_type();
        match tech {
            TechType::Archon_Warp => {
                if target_type != UnitType::Protoss_High_Templar {
                    return Err(Error::Incompatible_UnitType);
                }
                if target_unit.get_player() != unit.get_player() {
                    return Err(Error::Unit_Not_Owned);
                }
            }
            TechType::Dark_Archon_Meld => {
                if target_type != UnitType::Protoss_Dark_Templar {
                    return Err(Error::Incompatible_UnitType);
                }
                if target_unit.get_player() == unit.get_player() {
                    return Err(Error::Unit_Not_Owned);
                }
            }
            TechType::Consume => {
                if target_unit.get_player() != unit.get_player() {
                    return Err(Error::Unit_Not_Owned);
                }
                if target_type.get_race() != Race::Zerg || target_type == UnitType::Zerg_Larva {
                    return Err(Error::Unit_Not_Owned);
                }
                if target_type.get_race() != Race::Zerg || target_type == UnitType::Zerg_Larva {
                    return Err(Error::Incompatible_UnitType);
                }
            }
            TechType::Spawn_Broodlings => {
                if (!target_type.is_organic() && !target_type.is_mechanical())
                    || target_type.is_robotic()
                    || target_type.is_flyer()
                {
                    return Err(Error::Incompatible_UnitType);
                }
            }
            TechType::Lockdown => {
                if !target_type.is_mechanical() {
                    return Err(Error::Incompatible_UnitType);
                }
            }
            TechType::Healing => {
                if target_unit.get_hit_points() == target_type.max_hit_points() {
                    return Err(Error::Incompatible_State);
                }
                if !target_type.is_organic() || target_type.is_flyer() {
                    return Err(Error::Incompatible_UnitType);
                }
                if !target_unit.get_player().is_neutral()
                    && unit.get_player().is_enemy(&target_unit.get_player())
                {
                    return Err(Error::Invalid_Parameter);
                }
            }
            TechType::Mind_Control => {
                if target_unit.get_player() == unit.get_player() {
                    return Err(Error::Invalid_Parameter);
                }
                if target_type == UnitType::Protoss_Interceptor
                    || target_type == UnitType::Terran_Vulture_Spider_Mine
                    || target_type == UnitType::Zerg_Lurker_Egg
                    || target_type == UnitType::Zerg_Cocoon
                    || target_type == UnitType::Zerg_Larva
                    || target_type == UnitType::Zerg_Egg
                {
                    return Err(Error::Incompatible_UnitType);
                }
            }
            TechType::Feedback => {
                if !target_type.is_spellcaster() {
                    return Err(Error::Incompatible_UnitType);
                }
            }
            TechType::Infestation => {
                if target_type != UnitType::Terran_Command_Center
                    || target_unit.get_hit_points() >= 750
                    || target_unit.get_hit_points() <= 0
                {
                    return Err(Error::Invalid_Parameter);
                }
            }
            _ => (),
        }
        match tech {
            TechType::Archon_Warp | TechType::Dark_Archon_Meld => {
                if !unit.has_path(target_unit.get_position()) {
                    return Err(Error::Unreachable_Location);
                }
                if target_unit.is_hallucination() {
                    return Err(Error::Invalid_Parameter);
                }
                if target_unit.is_maelstrommed() {
                    return Err(Error::Incompatible_State);
                }
            }
            TechType::Parasite
            | TechType::Irradiate
            | TechType::Optical_Flare
            | TechType::Spawn_Broodlings
            | TechType::Lockdown
            | TechType::Defensive_Matrix
            | TechType::Hallucination
            | TechType::Healing
            | TechType::Restoration
            | TechType::Mind_Control
            | TechType::Consume
            | TechType::Feedback
            | TechType::Yamato_Gun => {
                if target_unit.is_stasised() {
                    return Err(Error::Incompatible_State);
                }
            }
            _ => (),
        }
        match tech {
            TechType::Yamato_Gun => {
                if target_unit.is_invincible() {
                    return Err(Error::Invalid_Parameter);
                }
            }
            TechType::Parasite
            | TechType::Irradiate
            | TechType::Optical_Flare
            | TechType::Spawn_Broodlings
            | TechType::Lockdown
            | TechType::Defensive_Matrix
            | TechType::Hallucination
            | TechType::Healing
            | TechType::Restoration
            | TechType::Mind_Control => {
                if target_unit.is_invincible() {
                    return Err(Error::Invalid_Parameter);
                }
                if target_type.is_building() {
                    return Err(Error::Incompatible_UnitType);
                }
            }
            TechType::Consume | TechType::Feedback => {
                if target_type.is_building() {
                    return Err(Error::Incompatible_UnitType);
                }
            }
            _ => (),
        }
        if target_unit == unit {
            return Err(Error::Invalid_Parameter);
        }
        Ok(true)
    }
}

pub trait CanUseTechPosition {
    fn can_use_tech_position(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanUseTechPosition for (TechType, bool, bool) {
    fn can_use_tech_position(&self, unit: &Unit) -> BwResult<bool> {
        let (tech, check_can_issue_command_type, check_commandability) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_use_tech_with_or_without_target(false)? {
            return Ok(false);
        }
        if !unit.can_use_tech_with_or_without_target((tech, false, false))? {
            return Ok(false);
        }
        if !tech.targets_position() {
            return Err(Error::Incompatible_TechType);
        }
        Ok(true)
    }
}

impl CanUseTechPosition for (TechType, Position, bool, bool, bool) {
    fn can_use_tech_position(&self, unit: &Unit) -> BwResult<bool> {
        let (
            tech,
            target,
            checks_targets_positions,
            check_can_issue_command_type,
            check_commandability,
        ) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_use_tech_with_or_without_target(false)? {
            return Ok(false);
        }
        if checks_targets_positions && !unit.can_use_tech_position((tech, false, false))? {
            return Ok(false);
        }

        if tech == TechType::Spider_Mines && !unit.has_path(target) {
            return Err(Error::Unreachable_Location);
        }

        Ok(true)
    }
}

pub trait CanUseTech {
    fn can_use_tech(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanUseTech for (TechType, Position, bool, bool, bool, bool) {
    fn can_use_tech(&self, unit: &Unit) -> BwResult<bool> {
        let (
            tech,
            target,
            _check_can_target_unit, // Not used in CPP code as well
            check_targets_type,
            check_can_issue_command_type,
            check_commandability,
        ) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.can_use_tech_position((
            tech,
            target,
            check_targets_type,
            check_can_issue_command_type,
            false,
        ))? {
            return Ok(false);
        }
        Ok(true)
    }
}

impl CanUseTech for (TechType, Option<&Unit<'_>>, bool, bool, bool, bool) {
    fn can_use_tech(&self, unit: &Unit) -> BwResult<bool> {
        let (
            tech,
            target,
            check_can_target_unit,
            check_targets_type,
            check_can_issue_command_type,
            check_commandability,
        ) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if let Some(target) = target {
            if !unit.can_use_tech_unit((
                tech,
                target,
                check_can_target_unit,
                check_targets_type,
                check_can_issue_command_type,
                false,
            ))? {
                return Ok(false);
            }
        } else if !unit.can_use_tech_without_target(tech, check_can_issue_command_type, false)? {
            return Ok(false);
        }
        Ok(true)
    }
}

pub trait CanPlaceCop {
    fn can_place_cop(&self, unit: &Unit) -> BwResult<bool>;
}

impl CanPlaceCop for bool {
    fn can_place_cop(&self, unit: &Unit) -> BwResult<bool> {
        if *self && !unit.can_command()? {
            return Ok(false);
        }

        if !unit.get_type().is_flag_beacon() {
            return Err(Error::Incompatible_UnitType);
        }
        if unit.get_buttonset() == 228 || unit.get_order() != Order::CTFCOPInit {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }
}

impl CanPlaceCop for (TilePosition, bool, bool) {
    fn can_place_cop(&self, unit: &Unit) -> BwResult<bool> {
        let (target, check_can_issue_command_type, check_commandability) = *self;

        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !unit.can_place_cop(false)? {
            return Ok(false);
        }

        if !unit
            .game
            .can_build_here(unit, target, unit.get_type(), true)?
        {
            return Ok(false);
        }
        Ok(true)
    }
}

impl<'a> Unit<'a> {
    pub fn can_build(&self, checker: impl CanBuild) -> BwResult<bool> {
        checker.can_build(self)
    }

    pub fn can_build_addon(&self, checker: impl CanBuildAddon) -> BwResult<bool> {
        checker.can_build_addon(self)
    }

    pub fn can_train(&self, checker: impl CanTrain) -> BwResult<bool> {
        checker.can_train(self)
    }

    pub fn can_morph(&self, checker: impl CanMorph) -> BwResult<bool> {
        checker.can_morph(self)
    }

    pub fn can_attack(&self, checker: impl CanAttack) -> BwResult<bool> {
        checker.can_attack(self)
    }

    pub fn can_attack_unit(&self, checker: impl CanAttackUnit) -> BwResult<bool> {
        checker.can_attack_unit(self)
    }

    pub fn can_attack_move(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if (self.get_type() != UnitType::Terran_Medic && !self.can_attack_unit(false)?)
            || !self.can_move(false)?
        {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn can_cancel_train(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.is_training() {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_cancel_train_slot(&self, checker: impl CanCancelTrainSlot) -> BwResult<bool> {
        checker.can_cancel_train_slot(self)
    }

    pub fn can_command(&self) -> Result<bool, Error> {
        if self.get_player() != self.game.self_().expect("Self to exist") {
            return Err(Error::Unit_Not_Owned);
        }

        if !self.exists() {
            return Err(Error::Unit_Does_Not_Exist);
        }

        if self.is_locked_down()
            || self.is_maelstrommed()
            || self.is_stasised()
            || !self.is_powered()
            || self.get_order() == Order::ZergBirth
            || self.is_loaded()
        {
            if !self.get_type().produces_larva() {
                return Err(Error::Unit_Busy);
            } else {
                for larva in self.get_larva() {
                    if larva.can_command().unwrap_or(false) {
                        return Ok(true);
                    }
                }
                return Err(Error::Unit_Busy);
            }
        }

        let u_type = self.get_type();
        if u_type == UnitType::Protoss_Interceptor
            || u_type == UnitType::Terran_Vulture_Spider_Mine
            || u_type == UnitType::Spell_Scanner_Sweep
            || u_type == UnitType::Special_Map_Revealer
        {
            return Err(Error::Incompatible_UnitType);
        }
        if self.is_completed()
            && (u_type == UnitType::Protoss_Pylon
                || u_type == UnitType::Terran_Supply_Depot
                || u_type.is_resource_container()
                || u_type == UnitType::Protoss_Shield_Battery
                || u_type == UnitType::Terran_Nuclear_Missile
                || u_type.is_powerup()
                || (u_type.is_special_building() && !u_type.is_flag_beacon()))
        {
            return Err(Error::Incompatible_State);
        }
        if !self.is_completed() && !u_type.is_building() && self.is_morphing() {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_follow(&self, checker: impl CanFollow) -> BwResult<bool> {
        checker.can_follow(self)
    }

    pub fn can_land(&self, checker: impl CanLand) -> BwResult<bool> {
        checker.can_land(self)
    }

    pub fn can_load(&self, checker: impl CanLoad) -> BwResult<bool> {
        checker.can_load(self)
    }

    pub fn can_gather(&self, checker: impl CanGather) -> BwResult<bool> {
        checker.can_gather(self)
    }

    pub fn can_place_cop(&self, checker: impl CanPlaceCop) -> BwResult<bool> {
        checker.can_place_cop(self)
    }

    pub fn can_repair(&self, checker: impl CanRepair) -> BwResult<bool> {
        checker.can_repair(self)
    }

    pub fn can_right_click_unit(&self, checker: impl CanRightClickUnit) -> BwResult<bool> {
        checker.can_right_click_unit(self)
    }

    pub fn can_set_rally_unit(&self, checker: impl CanSetRallyUnit) -> BwResult<bool> {
        checker.can_set_rally_unit(self)
    }

    pub fn can_unload(&self, checker: impl CanUnload) -> BwResult<bool> {
        checker.can_unload(self)
    }

    pub fn can_unload_all_position(&self, checker: impl CanUnloadAllPosition) -> BwResult<bool> {
        checker.can_unload_all_position(self)
    }

    pub fn can_use_tech(&self, checker: impl CanUseTech) -> BwResult<bool> {
        checker.can_use_tech(self)
    }

    pub fn can_use_tech_position(&self, checker: impl CanUseTechPosition) -> BwResult<bool> {
        checker.can_use_tech_position(self)
    }

    pub fn can_use_tech_unit(&self, checker: impl CanUseTechUnit) -> BwResult<bool> {
        checker.can_use_tech_unit(self)
    }

    pub fn can_use_tech_with_or_without_target(
        &self,
        checker: impl CanUseTechWithOrWithoutTarget,
    ) -> BwResult<bool> {
        checker.can_use_tech_with_or_without_target(self)
    }

    pub fn can_use_tech_without_target(
        &self,
        tech: TechType,
        check_can_issue_command_type: bool,
        check_commandability: bool,
    ) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !self.can_use_tech_with_or_without_target(false)? {
            return Ok(false);
        }

        if !self.can_use_tech_with_or_without_target((tech, false, false))? {
            return Ok(false);
        }

        if tech.targets_unit()
            || tech.targets_position()
            || tech == TechType::None
            || tech == TechType::Unknown
            || tech == TechType::Lurker_Aspect
        {
            return Err(Error::Incompatible_TechType);
        }
        Ok(true)
    }

    pub fn can_issue_command<C: Into<CanIssueCommandArg>>(&self, arg: C) -> Result<bool, Error> {
        let arg: CanIssueCommandArg = arg.into();
        if arg.check_commandability && !self.can_command()? {
            return Ok(false);
        }

        let ct = arg.c.get_type();
        if !self.can_issue_command_type(ct, false)? {
            return Ok(false);
        }

        let target = || {
            self.game
                .get_unit(arg.c.targetIndex as usize)
                .expect("Target to exist")
        };

        match ct {
            UnitCommandType::Attack_Move => Ok(true),
            UnitCommandType::Attack_Unit => {
                self.can_attack_unit((&target(), arg.check_can_target_unit, false, false))
            }
            UnitCommandType::Build => self.can_build((
                arg.c.get_unit_type(),
                TilePosition {
                    x: arg.c.x,
                    y: arg.c.y,
                },
                arg.check_can_build_unit_type,
                false,
                false,
            )),
            UnitCommandType::Build_Addon => {
                self.can_build_addon((arg.c.get_unit_type(), false, false))
            }
            UnitCommandType::Train => self.can_train((arg.c.get_unit_type(), false, false)),
            UnitCommandType::Morph => self.can_morph((arg.c.get_unit_type(), false, false)),
            UnitCommandType::Research => {
                self.game
                    .can_research((Some(self), arg.c.get_tech_type(), false))
            }
            UnitCommandType::Upgrade => {
                self.game
                    .can_upgrade((Some(self), arg.c.get_upgrade_type(), false))
            }
            UnitCommandType::Set_Rally_Position => Ok(true),
            UnitCommandType::Set_Rally_Unit => {
                self.can_set_rally_unit((&target(), arg.check_can_target_unit, false, false))
            }
            UnitCommandType::Move => Ok(true),
            UnitCommandType::Patrol => Ok(true),
            UnitCommandType::Hold_Position => Ok(true),
            UnitCommandType::Stop => Ok(true),
            UnitCommandType::Follow => {
                self.can_follow((&target(), arg.check_can_target_unit, false, false))
            }
            UnitCommandType::Gather => {
                self.can_gather((&target(), arg.check_can_target_unit, false, false))
            }
            UnitCommandType::Return_Cargo => Ok(true),
            UnitCommandType::Repair => {
                self.can_repair((&target(), arg.check_can_target_unit, false, false))
            }
            UnitCommandType::Burrow => Ok(true),
            UnitCommandType::Unburrow => Ok(true),
            UnitCommandType::Cloak => Ok(true),
            UnitCommandType::Decloak => Ok(true),
            UnitCommandType::Siege => Ok(true),
            UnitCommandType::Unsiege => Ok(true),
            UnitCommandType::Lift => Ok(true),
            UnitCommandType::Land => {
                self.can_land((arg.c.get_target_tile_position(), false, false))
            }
            UnitCommandType::Load => {
                self.can_load((&target(), arg.check_can_target_unit, false, false))
            }
            UnitCommandType::Unload => {
                self.can_unload((&target(), arg.check_can_target_unit, false, false, false))
            }
            UnitCommandType::Unload_All => Ok(true),
            UnitCommandType::Unload_All_Position => {
                self.can_unload_all_position((arg.c.get_target_position(), false, false))
            }
            UnitCommandType::Right_Click_Position => Ok(true),
            UnitCommandType::Right_Click_Unit => {
                self.can_right_click_unit((&target(), arg.check_can_target_unit, false, false))
            }
            UnitCommandType::Halt_Construction => Ok(true),
            UnitCommandType::Cancel_Construction => Ok(true),
            UnitCommandType::Cancel_Addon => Ok(true),
            UnitCommandType::Cancel_Train => Ok(true),
            UnitCommandType::Cancel_Train_Slot => {
                self.can_cancel_train_slot((arg.c.extra, false, false))
            }
            UnitCommandType::Cancel_Morph => Ok(true),
            UnitCommandType::Cancel_Research => Ok(true),
            UnitCommandType::Cancel_Upgrade => Ok(true),
            UnitCommandType::Use_Tech => {
                self.can_use_tech_without_target(arg.c.get_tech_type(), false, false)
            }
            UnitCommandType::Use_Tech_Unit => self.can_use_tech_unit((
                arg.c.get_tech_type(),
                &target(),
                arg.check_can_target_unit,
                arg.check_can_use_tech_unit_on_units,
                false,
                false,
            )),
            UnitCommandType::Use_Tech_Position => self.can_use_tech_position((
                arg.c.get_tech_type(),
                arg.c.get_target_position(),
                arg.check_can_use_tech_position_on_positions,
                false,
                false,
            )),
            UnitCommandType::Place_COP => {
                self.can_place_cop((arg.c.get_target_tile_position(), false, false))
            }
            _ => Ok(true),
        }
    }

    pub fn can_issue_command_type(
        &self,
        ct: UnitCommandType,
        check_commandability: bool,
    ) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }
        match ct {
            UnitCommandType::Attack_Move => self.can_attack_move(false),
            UnitCommandType::Attack_Unit => self.can_attack_unit(false),
            UnitCommandType::Build => self.can_build(false),
            UnitCommandType::Build_Addon => self.can_build_addon(false),
            UnitCommandType::Train => self.can_train(false),
            UnitCommandType::Morph => self.can_morph(false),
            UnitCommandType::Research => self.game.can_research((self, false)),
            UnitCommandType::Upgrade => self.game.can_upgrade((self, false)),
            UnitCommandType::Set_Rally_Position => self.can_set_rally_position(false),
            UnitCommandType::Set_Rally_Unit => self.can_set_rally_unit(false),
            UnitCommandType::Move => self.can_move(false),
            UnitCommandType::Patrol => self.can_patrol(false),
            UnitCommandType::Hold_Position => self.can_hold_position(false),
            UnitCommandType::Stop => self.can_stop(false),
            UnitCommandType::Follow => self.can_follow(false),
            UnitCommandType::Gather => self.can_gather(false),
            UnitCommandType::Return_Cargo => self.can_return_cargo(false),
            UnitCommandType::Repair => self.can_repair(false),
            UnitCommandType::Burrow => self.can_burrow(false),
            UnitCommandType::Unburrow => self.can_unburrow(false),
            UnitCommandType::Cloak => self.can_cloak(false),
            UnitCommandType::Decloak => self.can_decloak(false),
            UnitCommandType::Siege => self.can_siege(false),
            UnitCommandType::Unsiege => self.can_unsiege(false),
            UnitCommandType::Lift => self.can_lift(false),
            UnitCommandType::Land => self.can_land(false),
            UnitCommandType::Load => self.can_load(false),
            UnitCommandType::Unload => self.can_unload(false),
            UnitCommandType::Unload_All => self.can_unload_all(false),
            UnitCommandType::Unload_All_Position => self.can_unload_all_position(false),
            UnitCommandType::Right_Click_Position => self.can_right_click_position(false),
            UnitCommandType::Right_Click_Unit => self.can_right_click_unit(false),
            UnitCommandType::Halt_Construction => self.can_halt_construction(false),
            UnitCommandType::Cancel_Construction => self.can_cancel_construction(false),
            UnitCommandType::Cancel_Addon => self.can_cancel_addon(false),
            UnitCommandType::Cancel_Train => self.can_cancel_train(false),
            UnitCommandType::Cancel_Train_Slot => self.can_cancel_train_slot(false),
            UnitCommandType::Cancel_Morph => self.can_cancel_morph(false),
            UnitCommandType::Cancel_Research => self.can_cancel_research(false),
            UnitCommandType::Cancel_Upgrade => self.can_cancel_upgrade(false),
            UnitCommandType::Use_Tech
            | UnitCommandType::Use_Tech_Unit
            | UnitCommandType::Use_Tech_Position => self.can_use_tech_with_or_without_target(false),
            UnitCommandType::Place_COP => self.can_place_cop(false),
            _ => Ok(true),
        }
    }

    pub fn can_set_rally_position(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.get_type().can_produce() || !self.get_type().is_building() {
            return Err(Error::Incompatible_UnitType);
        }
        if self.is_lifted() {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_move(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.get_type().is_building() {
            if !self.is_interruptible() {
                return Err(Error::Unit_Busy);
            }
            if !self.get_type().can_move() {
                return Err(Error::Incompatible_UnitType);
            }
            if self.is_burrowed() {
                return Err(Error::Incompatible_State);
            }
            if self.get_order() == Order::ConstructingBuilding {
                return Err(Error::Unit_Busy);
            }
            if self.get_type() == UnitType::Zerg_Larva {
                return Err(Error::Incompatible_UnitType);
            }
        } else {
            if !self.get_type().is_flying_building() {
                return Err(Error::Incompatible_UnitType);
            }
            if !self.is_lifted() {
                return Err(Error::Incompatible_State);
            }
        }
        Ok(true)
    }

    pub fn can_patrol(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.can_move(false)? {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn can_return_cargo(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.get_type().is_building() && !self.is_interruptible() {
            return Err(Error::Unit_Busy);
        }
        if !self.get_type().is_worker() {
            return Err(Error::Incompatible_UnitType);
        }
        if !self.is_carrying_gas() && !self.is_carrying_minerals() {
            return Err(Error::Insufficient_Ammo);
        }
        if self.is_burrowed() {
            return Err(Error::Incompatible_State);
        }
        if self.get_order() == Order::ConstructingBuilding {
            return Err(Error::Unit_Busy);
        }
        Ok(true)
    }

    pub fn can_hold_position(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.get_type().is_building() {
            if !self.get_type().can_move() {
                return Err(Error::Incompatible_UnitType);
            }
            if self.is_burrowed() && self.get_type() != UnitType::Zerg_Lurker {
                return Err(Error::Incompatible_State);
            }
            if self.get_order() == Order::ConstructingBuilding {
                return Err(Error::Unit_Busy);
            }
            if self.get_type() == UnitType::Zerg_Larva {
                return Err(Error::Incompatible_UnitType);
            }
        } else {
            if !self.get_type().is_flying_building() {
                return Err(Error::Incompatible_UnitType);
            }
            if !self.is_lifted() {
                return Err(Error::Incompatible_State);
            }
        }

        if !self.is_completed() {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_stop(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.is_completed() {
            return Err(Error::Incompatible_State);
        }
        if self.is_burrowed() && self.get_type() != UnitType::Zerg_Lurker {
            return Err(Error::Incompatible_State);
        }
        if self.get_type().is_building()
            && !self.is_lifted()
            && self.get_type() != UnitType::Protoss_Photon_Cannon
            && self.get_type() != UnitType::Zerg_Sunken_Colony
            && self.get_type() != UnitType::Zerg_Spore_Colony
            && self.get_type() != UnitType::Terran_Missile_Turret
        {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_burrow(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.can_use_tech_without_target(TechType::Burrowing, true, false)? {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn can_unburrow(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.get_type().is_burrowable() {
            return Err(Error::Incompatible_UnitType);
        }
        if !self.is_burrowed() || self.get_order() == Order::Unburrowing {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_cloak(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if self.get_type().cloaking_tech() == TechType::None {
            return Err(Error::Incompatible_UnitType);
        }
        if self.get_secondary_order() != Order::Cloak {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_decloak(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if self.get_type().cloaking_tech() == TechType::None {
            return Err(Error::Incompatible_UnitType);
        }
        if self.get_secondary_order() != Order::Cloak {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_siege(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.can_use_tech_without_target(TechType::Tank_Siege_Mode, true, false)? {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn can_unsiege(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.is_sieged() {
            return Err(Error::Incompatible_State);
        }
        if self.get_order() == Order::Sieging || self.get_order() == Order::Unsieging {
            return Err(Error::Unit_Busy);
        }
        if self.is_hallucination() {
            return Err(Error::Incompatible_UnitType);
        }
        Ok(true)
    }

    pub fn can_lift(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.get_type().is_flying_building() {
            return Err(Error::Incompatible_UnitType);
        }
        if self.is_lifted() {
            return Err(Error::Incompatible_State);
        }
        if !self.is_completed() {
            return Err(Error::Incompatible_State);
        }
        if !self.is_idle() {
            return Err(Error::Unit_Busy);
        }
        Ok(true)
    }

    pub fn can_unload_with_or_without_target(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.get_type().is_building() && !self.is_interruptible() {
            return Err(Error::Unit_Busy);
        }
        if self.get_loaded_units().is_empty() {
            return Err(Error::Unit_Does_Not_Exist);
        }
        if self.get_type() == UnitType::Zerg_Overlord
            && self
                .game
                .self_()
                .expect("Player self to exist")
                .get_upgrade_level(UpgradeType::Ventral_Sacs)
                == 0
        {
            return Err(Error::Insufficient_Tech);
        }
        if self.get_type().space_provided() <= 0 {
            return Err(Error::Incompatible_UnitType);
        }

        Ok(true)
    }

    pub fn can_unload_at_position(
        &self,
        targ_drop_pos: Position,
        check_can_issue_command_type: bool,
        check_commandability: bool,
    ) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if check_can_issue_command_type && !self.can_unload_with_or_without_target(false)? {
            return Ok(false);
        }

        if self.get_type() != UnitType::Terran_Bunker {
            let wp = targ_drop_pos.to_walk_position();
            if !wp.is_valid() {
                return Err(Error::Invalid_Tile_Position);
            } else if !self.game.is_walkable(wp) {
                return Err(Error::Unreachable_Location);
            }
        }
        Ok(true)
    }

    pub fn can_unload_all(&self, check_commandability: bool) -> BwResult<bool> {
        self.can_unload_at_position(self.get_position(), true, check_commandability)
    }

    pub fn can_right_click(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.can_right_click_position(false).unwrap_or(false)
            && !self.can_right_click_unit(false)?
        {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn can_right_click_position(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.get_type().is_building() && !self.is_interruptible() {
            return Err(Error::Unit_Busy);
        }
        if !self.can_move(false).unwrap_or(false)
            && !self.can_set_rally_position(false).unwrap_or(false)
        {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_halt_construction(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if self.get_order() != Order::ConstructingBuilding {
            return Err(Error::Incompatible_State);
        }

        Ok(true)
    }

    pub fn can_cancel_construction(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if self.get_type().is_building() {
            return Err(Error::Incompatible_UnitType);
        }
        if self.is_completed()
            || (self.get_type() == UnitType::Zerg_Nydus_Canal && self.get_nydus_exit().is_some())
        {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_cancel_addon(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.get_addon().map(|a| a.is_completed()).unwrap_or(false) {
            return Err(Error::Incompatible_UnitType);
        }
        Ok(true)
    }

    pub fn can_cancel_research(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if self.get_order() == Order::ResearchTech {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_cancel_upgrade(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if self.get_order() != Order::Upgrade {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_cancel_morph(&self, check_commandability: bool) -> BwResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.is_morphing()
            || (!self.is_completed()
                && self.get_type() == UnitType::Zerg_Nydus_Canal
                && self.get_nydus_exit().is_some())
        {
            return Err(Error::Incompatible_State);
        }
        if self.is_hallucination() {
            return Err(Error::Incompatible_UnitType);
        }
        Ok(true)
    }
}

pub trait CanResearch {
    fn can_research(&self, game: &Game) -> BwResult<bool>;
}

impl CanResearch for (&Unit<'_>, bool) {
    fn can_research(&self, _: &Game) -> BwResult<bool> {
        let (unit, check_commandability) = *self;
        if check_commandability && !unit.can_command()? {
            return Ok(false);
        }

        if unit.is_lifted() || !unit.is_idle() || unit.is_completed() {
            return Err(Error::Unit_Busy);
        }
        Ok(true)
    }
}

impl CanResearch for (Option<&Unit<'_>>, TechType, bool) {
    fn can_research(&self, game: &Game) -> BwResult<bool> {
        let self_ = game.self_().ok_or(Error::Unit_Not_Owned)?;
        let (unit, type_, check_can_issue_command_type) = *self;

        if let Some(unit) = unit {
            if Some(unit.get_player()) != game.self_() {
                return Err(Error::Unit_Not_Owned);
            }
            if !unit.get_type().is_successor_of(type_.what_researches()) {
                return Err(Error::Incompatible_UnitType);
            }
            if check_can_issue_command_type && unit.is_lifted()
                || !unit.is_idle()
                || !unit.is_completed()
            {
                return Err(Error::Unit_Busy);
            }
        }
        if self_.is_researching(type_) {
            return Err(Error::Currently_Researching);
        }
        if self_.has_researched(type_) {
            return Err(Error::Already_Researched);
        }
        if self_.is_research_available(type_) {
            return Err(Error::Access_Denied);
        }
        if self_.minerals() < type_.mineral_price() {
            return Err(Error::Insufficient_Minerals);
        }
        if self_.gas() < type_.gas_price() {
            return Err(Error::Insufficient_Gas);
        }
        if !self_.has_unit_type_requirement(type_.required_unit(), 1) {
            return Err(Error::Insufficient_Tech);
        }
        Ok(true)
    }
}

pub trait CanTargetUnit {
    fn can_target_unit(&self, game: &Game) -> BwResult<bool>;
}

impl CanTargetUnit for &Unit<'_> {
    fn can_target_unit(&self, _: &Game) -> BwResult<bool> {
        let target_unit = self;
        if !target_unit.exists() {
            return Err(Error::Unit_Does_Not_Exist);
        }
        if !target_unit.is_visible() && !self.game.is_flag_enabled(Flag::CompleteMapInformation) {
            return Ok(false);
        }
        if !target_unit.is_completed()
            && !target_unit.get_type().is_building()
            && !target_unit.is_morphing()
            && target_unit.get_type() != UnitType::Protoss_Archon
            && target_unit.get_type() != UnitType::Protoss_Dark_Archon
        {
            return Err(Error::Incompatible_State);
        }
        if target_unit.get_type() == UnitType::Spell_Scanner_Sweep
            || target_unit.get_type() == UnitType::Spell_Dark_Swarm
            || target_unit.get_type() == UnitType::Spell_Disruption_Web
            || target_unit.get_type() == UnitType::Special_Map_Revealer
        {
            return Err(Error::Incompatible_UnitType);
        }
        Ok(true)
    }
}

impl CanTargetUnit for (&Unit<'_>, &Unit<'_>, bool) {
    fn can_target_unit(&self, game: &Game) -> BwResult<bool> {
        let (this_unit, target_unit, check_commandability) = *self;
        if check_commandability && !this_unit.can_command()? {
            return Ok(false);
        }

        if !game.can_target_unit(target_unit)? {
            return Ok(false);
        }
        Ok(true)
    }
}

pub trait CanUpgrade {
    fn can_upgrade(&self, game: &Game) -> BwResult<bool>;
}

impl CanUpgrade for (&Unit<'_>, bool) {
    fn can_upgrade(&self, _: &Game) -> BwResult<bool> {
        let (this_unit, check_commandability) = *self;
        if check_commandability && !this_unit.can_command()? {
            return Ok(false);
        }

        if this_unit.is_lifted() || !this_unit.is_idle() || !this_unit.is_completed() {
            return Err(Error::Unit_Busy);
        }
        Ok(true)
    }
}

impl CanUpgrade for (Option<&Unit<'_>>, UpgradeType, bool) {
    fn can_upgrade(&self, game: &Game) -> BwResult<bool> {
        let (this_unit, type_, check_can_issue_command_type) = *self;
        let self_ = game.self_().ok_or(Error::Unit_Not_Owned)?;

        if let Some(this_unit) = this_unit {
            if this_unit.get_player() != self_ {
                return Err(Error::Unit_Not_Owned);
            }
            if !this_unit.get_type().is_successor_of(type_.what_upgrades()) {
                return Err(Error::Incompatible_UnitType);
            }
            if check_can_issue_command_type
                && (this_unit.is_lifted() || !this_unit.is_idle() || !this_unit.is_completed())
            {
                return Err(Error::Unit_Busy);
            }
        }
        let next_lvl = self_.get_upgrade_level(type_) + 1;

        if !self_.has_unit_type_requirement(type_.what_upgrades(), 1) {
            return Err(Error::Unit_Does_Not_Exist);
        }

        if !self_.has_unit_type_requirement(type_.whats_required(next_lvl), 1) {
            return Err(Error::Insufficient_Tech);
        }

        if self_.is_upgrading(type_) {
            return Err(Error::Currently_Upgrading);
        }

        if self_.get_upgrade_level(type_) >= self_.get_max_upgrade_level(type_) {
            return Err(Error::Fully_Upgraded);
        }

        if self_.minerals() < type_.mineral_price(next_lvl) {
            return Err(Error::Insufficient_Minerals);
        }

        if self_.gas() < type_.gas_price(next_lvl) {
            return Err(Error::Insufficient_Gas);
        }

        Ok(true)
    }
}

impl<'a> Game<'a> {
    pub fn can_research(&self, checker: impl CanResearch) -> BwResult<bool> {
        checker.can_research(self)
    }

    pub fn can_upgrade(&self, checker: impl CanUpgrade) -> BwResult<bool> {
        checker.can_upgrade(self)
    }

    pub(crate) fn can_target_unit(&self, checker: impl CanTargetUnit) -> BwResult<bool> {
        checker.can_target_unit(self)
    }
}
