use crate::*;

pub trait CanAttackUnit {
    fn can_attack_unit(&self, unit: &Unit) -> BWResult<bool>;
}

pub trait CanAttack {
    fn can_attack(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanAttack for bool {
    fn can_attack(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_attack_unit(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_attack(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_attack(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_attack_unit(&self, unit: &Unit) -> BWResult<bool> {
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

pub trait CanBuild {
    fn can_build(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanBuild for bool {
    fn can_build(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_build(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_build(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_build_addon(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanBuildAddon for bool {
    fn can_build_addon(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_build_addon(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_train(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanTrain for bool {
    fn can_train(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_train(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_morph(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanMorph for bool {
    fn can_morph(&self, unit: &Unit) -> BWResult<bool> {
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
            if unit.get_larva().iter().any(|u| {
                !u.is_constructing() && u.is_completed() && u.can_command().unwrap_or(false)
            }) {
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
    fn can_morph(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_follow(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanFollow for bool {
    fn can_follow(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_follow(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_set_rally_unit(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanSetRallyUnit for bool {
    fn can_set_rally_unit(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_set_rally_unit(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_gather(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanGather for bool {
    fn can_gather(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_gather(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_repair(&self, unit: &Unit) -> BWResult<bool>;
}
impl CanRepair for bool {
    fn can_repair(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_repair(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_land(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanLand for bool {
    fn can_land(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_land(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_load(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanLoad for bool {
    fn can_load(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_load(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_unload(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanUnload for bool {
    fn can_unload(&self, unit: &Unit) -> BWResult<bool> {
        unit.can_unload_at_position(unit.get_position(), true, *self)
    }
}

impl CanUnload for (&Unit<'_>, bool, bool, bool, bool) {
    fn can_unload(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_unload_all_position(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanUnloadAllPosition for bool {
    fn can_unload_all_position(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_unload_all_position(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_right_click_unit(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanRightClickUnit for bool {
    fn can_right_click_unit(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_right_click_unit(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_cancel_train_slot(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanCancelTrainSlot for bool {
    fn can_cancel_train_slot(&self, unit: &Unit) -> BWResult<bool> {
        unit.can_cancel_train(*self)
    }
}

impl CanCancelTrainSlot for (i32, bool, bool) {
    fn can_cancel_train_slot(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_use_tech_with_or_without_target(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanUseTechWithOrWithoutTarget for bool {
    fn can_use_tech_with_or_without_target(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_use_tech_with_or_without_target(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_use_tech_unit(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanUseTechUnit for (TechType, bool, bool) {
    fn can_use_tech_unit(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_use_tech_unit(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_use_tech_position(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanUseTechPosition for (TechType, bool, bool) {
    fn can_use_tech_position(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_use_tech_position(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_use_tech(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanUseTech for (TechType, Position, bool, bool, bool, bool) {
    fn can_use_tech(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_use_tech(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_place_cop(&self, unit: &Unit) -> BWResult<bool>;
}

impl CanPlaceCop for bool {
    fn can_place_cop(&self, unit: &Unit) -> BWResult<bool> {
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
    fn can_place_cop(&self, unit: &Unit) -> BWResult<bool> {
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

impl<'a> Unit<'a> {}

pub trait CanResearch {
    fn can_research(&self, game: &Game) -> BWResult<bool>;
}

impl CanResearch for (&Unit<'_>, bool) {
    fn can_research(&self, _: &Game) -> BWResult<bool> {
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
    fn can_research(&self, game: &Game) -> BWResult<bool> {
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
    fn can_target_unit(&self, game: &Game) -> BWResult<bool>;
}

impl CanTargetUnit for &Unit<'_> {
    fn can_target_unit(&self, _: &Game) -> BWResult<bool> {
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
    fn can_target_unit(&self, game: &Game) -> BWResult<bool> {
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
    fn can_upgrade(&self, game: &Game) -> BWResult<bool>;
}

impl CanUpgrade for (&Unit<'_>, bool) {
    fn can_upgrade(&self, _: &Game) -> BWResult<bool> {
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
    fn can_upgrade(&self, game: &Game) -> BWResult<bool> {
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
    pub fn can_research(&self, checker: impl CanResearch) -> BWResult<bool> {
        checker.can_research(self)
    }

    pub fn can_upgrade(&self, checker: impl CanUpgrade) -> BWResult<bool> {
        checker.can_upgrade(self)
    }

    pub(crate) fn can_target_unit(&self, checker: impl CanTargetUnit) -> BWResult<bool> {
        checker.can_target_unit(self)
    }
}
