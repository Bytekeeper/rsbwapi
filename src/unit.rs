use crate::player::Player;
use crate::predicate::{IntoPredicate, Predicate};
use crate::projected::Projected;

use crate::*;
use bwapi_wrapper::*;
use std::{borrow::Borrow, convert::From, fmt};

pub type UnitId = usize;

#[derive(Clone, Copy, Debug)]
pub(crate) struct UnitInfo {
    pub initial_hit_points: i32,
    pub initial_resources: i32,
    pub initial_position: Position,
    pub initial_type: UnitType,
    pub last_command_frame: i32,
}

impl UnitInfo {
    pub(crate) fn new(data: &BWAPI_UnitData) -> Self {
        Self {
            initial_hit_points: data.hitPoints,
            initial_resources: data.resources,
            initial_position: Position {
                x: data.positionX,
                y: data.positionY,
            },
            initial_type: UnitType::new(data.type_),
            last_command_frame: 0,
        }
    }
}

#[derive(Clone)]
pub struct Unit {
    id: UnitId,
    pub(crate) inner: Projected<Game, BWAPI_UnitData>,
}

impl From<Unit> for UnitId {
    fn from(unit: Unit) -> Self {
        unit.id
    }
}

impl fmt::Debug for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Unit")
            .field("id", &self.id)
            .field("type", &self.get_type())
            .field("position", &self.get_position())
            .finish()
    }
}

impl Unit {
    pub fn get_type(&self) -> UnitType {
        UnitType::new(self.inner.type_)
    }
    pub(crate) fn new(id: UnitId, game: Game, data: *const BWAPI_UnitData) -> Self {
        Unit {
            id,
            inner: unsafe { Projected::new(game, data) },
        }
    }

    fn info(&self) -> UnitInfo {
        self.inner.game().inner.unit_infos.borrow()[self.id]
    }

    fn set_info(&self, unit_info: UnitInfo) {
        (*self.inner.game().inner.unit_infos.borrow_mut())[self.id] = unit_info;
    }

    pub(crate) fn get_buttonset(&self) -> i32 {
        self.inner.buttonset
    }

    pub fn get_closest_unit<R: Into<Option<u32>>>(
        &self,
        pred: impl Fn(&Unit) -> bool,
        radius: R,
    ) -> Option<Unit> {
        self.inner
            .game()
            .get_closest_unit(self.get_position(), pred, radius)
    }

    pub fn is_accelerating(&self) -> bool {
        self.inner.isAccelerating
    }

    pub fn is_attacking(&self) -> bool {
        self.inner.isAttacking
    }

    pub fn is_attack_game(&self) -> bool {
        self.inner.isAttackFrame
    }

    pub fn get_acid_spore_count(&self) -> i32 {
        self.inner.acidSporeCount
    }

    pub fn get_addon(&self) -> Option<Unit> {
        self.inner.game().get_unit(self.inner.addon as usize)
    }

    pub fn get_air_weapon_cooldown(&self) -> i32 {
        self.inner.airWeaponCooldown
    }

    pub fn get_angle(&self) -> f64 {
        self.inner.angle
    }

    pub fn get_bottom(&self) -> i32 {
        self.get_position().y + self.get_type().dimension_down()
    }

    pub fn get_build_type(&self) -> UnitType {
        UnitType::new(self.inner.buildType)
    }

    pub fn get_build_unit(&self) -> Option<Unit> {
        self.inner.game().get_unit(self.inner.buildUnit as usize)
    }

    pub fn get_carrier(&self) -> Option<Unit> {
        self.inner.game().get_unit(self.inner.carrier as usize)
    }

    pub fn get_defense_matrix_points(&self) -> i32 {
        self.inner.defenseMatrixPoints
    }

    pub fn get_defense_matrix_timer(&self) -> i32 {
        self.inner.defenseMatrixTimer
    }

    pub fn get_distance(&self, target: impl Borrow<Unit>) -> i32 {
        let target = target.borrow();
        if !self.exists() || !target.exists() {
            return i32::MAX;
        }

        if self == target {
            return 0;
        }

        let left = target.get_left() - 1;
        let top = target.get_top() - 1;
        let right = target.get_right() + 1;
        let bottom = target.get_bottom() + 1;

        let mut x_dist = self.get_left() - right;
        if x_dist < 0 {
            x_dist = (left - self.get_right()).max(0);
        }

        let mut y_dist = self.get_top() - bottom;
        if y_dist < 0 {
            y_dist = (top - self.get_bottom()).max(0);
        }

        ORIGIN.get_approx_distance(Position {
            x: x_dist,
            y: y_dist,
        })
    }

    pub fn get_energy(&self) -> i32 {
        self.inner.energy
    }

    pub fn get_ensnare_timer(&self) -> i32 {
        self.inner.ensnareTimer
    }

    pub fn get_ground_weapon_cooldown(&self) -> i32 {
        self.inner.groundWeaponCooldown
    }

    pub fn get_hatchery(&self) -> Option<Unit> {
        self.inner.game().get_unit(self.inner.hatchery as usize)
    }

    pub fn get_hit_points(&self) -> i32 {
        self.inner.hitPoints
    }

    pub fn get_initial_hit_points(&self) -> i32 {
        self.info().initial_hit_points
    }

    pub fn get_initial_resources(&self) -> i32 {
        self.info().initial_resources
    }

    pub fn get_initial_tile_position(&self) -> TilePosition {
        (self.get_initial_position() - self.get_initial_type().tile_size().to_position() / 2)
            .to_tile_position()
    }

    pub fn get_initial_position(&self) -> Position {
        self.info().initial_position
    }

    pub fn get_initial_type(&self) -> UnitType {
        self.info().initial_type
    }

    pub fn get_interceptor_count(&self) -> i32 {
        self.inner.interceptorCount
    }

    pub fn get_interceptors(&self) -> Vec<Unit> {
        if self.get_type() != UnitType::Protoss_Carrier
            && self.get_type() != UnitType::Hero_Gantrithor
        {
            return vec![];
        }
        self.inner
            .game()
            .get_all_units()
            .iter()
            .filter(|u| u.get_carrier().as_ref() == Some(self))
            .cloned()
            .collect()
    }

    pub fn get_irradiate_timer(&self) -> i32 {
        self.inner.irradiateTimer
    }

    pub fn get_kill_count(&self) -> i32 {
        self.inner.killCount
    }

    pub fn get_larva(&self) -> Vec<Unit> {
        if !self.get_type().produces_larva() {
            return vec![];
        }
        self.inner
            .game()
            .get_all_units()
            .iter()
            .filter(|u| u.get_hatchery().as_ref() == Some(self))
            .cloned()
            .collect()
    }

    pub fn get_last_attacking_player(&self) -> Option<Player> {
        self.inner
            .game()
            .get_player(self.inner.lastAttackerPlayer as usize)
    }

    pub fn get_left(&self) -> i32 {
        self.get_position().x - self.get_type().dimension_left()
    }

    pub fn get_loaded_units(&self) -> Vec<Unit> {
        self.inner
            .game()
            .get_all_units()
            .iter()
            .filter(|u| {
                if let Some(transport) = u.get_transport() {
                    transport == *self
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    pub fn get_lockdown_timer(&self) -> i32 {
        self.inner.lockdownTimer
    }

    pub fn get_maelstrom_timer(&self) -> i32 {
        self.inner.maelstromTimer
    }

    pub fn get_nydus_exit(&self) -> Option<Unit> {
        self.inner.game().get_unit(self.inner.nydusExit as usize)
    }

    pub fn get_order(&self) -> Order {
        Order::new(self.inner.order)
    }

    pub fn get_order_target(&self) -> Option<Unit> {
        self.inner.game().get_unit(self.inner.orderTarget as usize)
    }

    pub fn get_order_target_position(&self) -> Option<Position> {
        Position::new_checked(
            self.inner.game(),
            self.inner.orderTargetPositionX,
            self.inner.orderTargetPositionY,
        )
    }

    pub fn get_plague_timer(&self) -> i32 {
        self.inner.plagueTimer
    }

    pub fn get_position(&self) -> Position {
        Position {
            x: self.inner.positionX,
            y: self.inner.positionY,
        }
    }

    pub fn get_power_up(&self) -> Option<Unit> {
        self.inner.game().get_unit(self.inner.powerUp as usize)
    }

    pub fn get_rally_position(&self) -> Option<Position> {
        if self.inner.rallyPositionX < 0 {
            None
        } else {
            Some(Position {
                x: self.inner.rallyPositionX,
                y: self.inner.rallyPositionY,
            })
        }
    }

    pub fn get_rally_unit(&self) -> Option<Unit> {
        self.inner.game().get_unit(self.inner.rallyUnit as usize)
    }

    pub fn get_remaining_build_time(&self) -> i32 {
        self.inner.remainingBuildTime
    }

    pub fn get_remaining_research_time(&self) -> i32 {
        self.inner.remainingResearchTime
    }

    pub fn get_remaining_train_time(&self) -> i32 {
        self.inner.remainingTrainTime
    }

    pub fn get_remove_timer(&self) -> i32 {
        self.inner.removeTimer
    }

    pub fn get_replay_id(&self) -> i32 {
        self.inner.replayID
    }

    pub fn get_resource_group(&self) -> i32 {
        self.inner.resourceGroup
    }

    pub fn get_resources(&self) -> i32 {
        self.inner.resources
    }

    pub fn get_right(&self) -> i32 {
        self.get_position().x + self.get_type().dimension_right()
    }

    pub fn get_scarab_count(&self) -> i32 {
        self.inner.scarabCount
    }

    pub fn get_secondary_order(&self) -> Order {
        Order::new(self.inner.secondaryOrder)
    }

    pub fn get_shields(&self) -> i32 {
        self.inner.shields
    }

    pub fn get_space_remaining(&self) -> i32 {
        self.get_type().space_provided()
            - self
                .get_loaded_units()
                .iter()
                .map(|u| u.get_type().space_required())
                .sum::<i32>()
    }

    pub fn get_spell_cooldown(&self) -> i32 {
        self.inner.spellCooldown
    }

    pub fn get_spider_mine_count(&self) -> i32 {
        self.inner.spiderMineCount
    }

    pub fn get_stasis_timer(&self) -> i32 {
        self.inner.stasisTimer
    }

    pub fn get_stim_timer(&self) -> i32 {
        self.inner.stimTimer
    }

    pub fn get_target(&self) -> Option<Unit> {
        self.inner.game().get_unit(self.inner.target as usize)
    }

    pub fn get_target_position(&self) -> Option<Position> {
        Position::new_checked(
            self.inner.game(),
            self.inner.targetPositionX,
            self.inner.targetPositionY,
        )
    }

    pub fn get_tech(&self) -> TechType {
        TechType::new(self.inner.tech)
    }

    pub fn get_tile_position(&self) -> TilePosition {
        (self.get_position()
            - Position {
                x: self.get_type().tile_width(),
                y: self.get_type().tile_height(),
            } * 32
                / 2)
        .to_tile_position()
    }

    pub fn get_top(&self) -> i32 {
        self.get_position().y - self.get_type().dimension_up()
    }

    pub fn get_training_queue(&self) -> Vec<UnitType> {
        (0..self.inner.trainingQueueCount as usize)
            .map(|i| self.inner.trainingQueue[i])
            .map(UnitType::new)
            .collect()
    }

    pub fn get_transport(&self) -> Option<Unit> {
        self.inner.game().get_unit(self.inner.transport as usize)
    }

    pub fn get_upgrade(&self) -> UpgradeType {
        UpgradeType::new(self.inner.upgrade)
    }

    pub fn get_units_in_radius<Pred: IntoPredicate<Unit>>(
        &self,
        radius: i32,
        pred: Pred,
    ) -> Vec<Unit> {
        self.inner
            .game()
            .get_units_in_radius(self.get_position(), radius, pred)
    }

    pub fn get_units_in_weapon_range<Pred: IntoPredicate<Unit>>(
        &self,
        weapon: WeaponType,
        pred: Pred,
    ) -> Vec<Unit> {
        if !self.exists() {
            return vec![];
        }

        let pred = pred.into_predicate();
        let max = self.get_player().weapon_max_range(weapon);
        self.inner.game().get_units_in_rectangle(
            (self.get_left() - max, self.get_top() - max),
            (self.get_right() + max, self.get_bottom() * max),
            |u: &Unit| -> bool {
                if u == self || u.is_invincible() {
                    return false;
                }

                let dist = self.get_distance(u);
                if weapon.min_range() > 0 && dist < weapon.min_range() || dist > max {
                    return false;
                }

                let ut = u.get_type();
                if (weapon.targets_own() && u.get_player() != self.get_player())
                    || (!weapon.targets_air() && !u.is_flying())
                    || (!weapon.targets_ground() && u.is_flying())
                    || (weapon.targets_mechanical() && ut.is_mechanical())
                    || (weapon.targets_organic() && ut.is_organic())
                    || (weapon.targets_non_building() && !ut.is_building())
                    || (weapon.targets_non_robotic() && !ut.is_robotic())
                    || (weapon.targets_org_or_mech() && (ut.is_organic() || ut.is_mechanical()))
                {
                    return false;
                }
                pred.test(u)
            },
        )
    }

    pub fn get_velocity(&self) -> Vector2D {
        Vector2D {
            x: self.inner.velocityX,
            y: self.inner.velocityY,
        }
    }

    pub fn has_nuke(&self) -> bool {
        self.inner.hasNuke
    }

    pub fn is_blind(&self) -> bool {
        self.inner.isBlind
    }

    pub fn is_braking(&self) -> bool {
        self.inner.isBraking
    }

    pub fn is_burrowed(&self) -> bool {
        self.inner.isBurrowed
    }

    pub fn is_carrying_gas(&self) -> bool {
        self.inner.carryResourceType == 1
    }

    pub fn is_carrying_minerals(&self) -> bool {
        self.inner.carryResourceType == 2
    }

    pub fn is_constructing(&self) -> bool {
        self.inner.isConstructing
    }

    pub fn is_defense_matrixed(&self) -> bool {
        self.inner.defenseMatrixTimer > 0
    }

    pub fn is_cloaked(&self) -> bool {
        self.inner.isCloaked
    }

    pub fn is_detected(&self) -> bool {
        self.inner.isDetected
    }

    pub fn is_ensnared(&self) -> bool {
        self.inner.ensnareTimer > 0
    }

    pub fn is_flying(&self) -> bool {
        self.get_type().is_flyer() || self.is_lifted()
    }

    pub fn is_following(&self) -> bool {
        self.get_order() == Order::Follow
    }

    pub fn is_gathering_gas(&self) -> bool {
        if !self.inner.isGathering {
            return false;
        }

        if self.get_order() != Order::Harvest1
            && self.get_order() != Order::Harvest2
            && self.get_order() != Order::MoveToGas
            && self.get_order() != Order::WaitForGas
            && self.get_order() != Order::HarvestGas
            && self.get_order() != Order::ReturnGas
            && self.get_order() != Order::ResetCollision
        {
            return false;
        }

        if self.get_order() == Order::ResetCollision {
            return self.inner.carryResourceType == 1;
        }

        //return true if BWOrder is WaitForGas, HarvestGas, or ReturnGas
        if self.get_order() == Order::WaitForGas
            || self.get_order() == Order::HarvestGas
            || self.get_order() == Order::ReturnGas
        {
            return true;
        }

        //if BWOrder is MoveToGas, Harvest1, or Harvest2 we need to do some additional checks to make sure the unit is really gathering
        if let Some(targ) = self.get_target() {
            if targ.exists()
                && targ.is_completed()
                && targ.get_player() == self.get_player()
                && targ.get_type() != UnitType::Resource_Vespene_Geyser
                && (targ.get_type().is_refinery() || targ.get_type().is_resource_depot())
            {
                return true;
            }
        }
        if let Some(targ) = self.get_order_target() {
            if targ.exists()
                && targ.is_completed()
                && targ.get_player() == self.get_player()
                && targ.get_type() != UnitType::Resource_Vespene_Geyser
                && (targ.get_type().is_refinery() || targ.get_type().is_resource_depot())
            {
                return true;
            }
        }
        false
    }

    pub fn is_gathering_minerals(&self) -> bool {
        if !self.inner.isGathering {
            return false;
        }
        if self.get_order() != Order::Harvest1
            && self.get_order() != Order::Harvest2
            && self.get_order() != Order::MoveToMinerals
            && self.get_order() != Order::WaitForMinerals
            && self.get_order() != Order::MiningMinerals
            && self.get_order() != Order::ReturnMinerals
            && self.get_order() != Order::ResetCollision
        {
            return false;
        }

        if self.get_order() == Order::ResetCollision {
            return self.inner.carryResourceType == 2;
        }

        //return true if BWOrder is WaitForMinerals, MiningMinerals, or ReturnMinerals
        if self.get_order() == Order::WaitForMinerals
            || self.get_order() == Order::MiningMinerals
            || self.get_order() == Order::ReturnMinerals
        {
            return true;
        }

        //if BWOrder is MoveToMinerals, Harvest1, or Harvest2 we need to do some additional checks to make sure the unit is really gathering
        if let Some(target) = self.get_target() {
            if target.exists()
                && (target.get_type().is_mineral_field()
                    || (target.is_completed()
                        && target.get_player() == self.get_player()
                        && target.get_type().is_resource_depot()))
            {
                return true;
            }
        }
        if let Some(order_target) = self.get_order_target() {
            if order_target.exists()
                && (order_target.get_type().is_mineral_field()
                    || (order_target.is_completed()
                        && order_target.get_player() == self.get_player()
                        && order_target.get_type().is_resource_depot()))
            {
                return true;
            }
        }
        false
    }

    pub fn is_hallucination(&self) -> bool {
        self.inner.isHallucination
    }

    pub fn is_holding_position(&self) -> bool {
        self.get_order() == Order::HoldPosition
    }

    pub fn is_idle(&self) -> bool {
        self.inner.isIdle
    }

    pub fn is_interruptible(&self) -> bool {
        self.inner.isInterruptible
    }

    pub fn is_invincible(&self) -> bool {
        self.inner.isInvincible
    }

    pub fn has_weapon_against(&self, target: &Unit) -> bool {
        self.weapon_type_vs(target) != WeaponType::None
    }

    pub fn weapon_type_vs(&self, target: &Unit) -> WeaponType {
        let this_type = self.get_type();
        if target.is_flying() {
            this_type.air_weapon()
        } else {
            this_type.ground_weapon()
        }
    }

    pub fn weapon_range_vs(&self, target: &Unit) -> i32 {
        self.weapon_type_vs(target).max_range()
    }

    pub fn cooldown(&self) -> i32 {
        self.inner
            .groundWeaponCooldown
            .max(self.inner.airWeaponCooldown)
    }

    pub fn is_in_weapon_range(&self, target: &Unit) -> bool {
        if !self.exists() || !target.exists() || self == target {
            return false;
        }

        let wpn = self.weapon_type_vs(target);

        if wpn == WeaponType::None || wpn == WeaponType::Unknown {
            return false;
        }

        let min_range = wpn.min_range();
        let max_range = wpn.max_range();
        let distance = self.get_distance(target);
        (min_range == 0 || min_range < distance) && distance <= max_range
    }

    pub fn is_irradiated(&self) -> bool {
        self.inner.irradiateTimer != 0
    }

    pub fn is_lifted(&self) -> bool {
        self.inner.isLifted
    }

    pub fn is_loaded(&self) -> bool {
        self.get_transport().is_some()
    }

    pub fn is_locked_down(&self) -> bool {
        self.inner.lockdownTimer != 0
    }

    pub fn is_maelstrommed(&self) -> bool {
        self.inner.maelstromTimer != 0
    }

    pub fn is_parasited(&self) -> bool {
        self.inner.isParasited
    }

    pub fn is_patrolling(&self) -> bool {
        self.get_order() == Order::Patrol
    }

    pub fn is_plagued(&self) -> bool {
        self.inner.plagueTimer != 0
    }

    pub fn is_powered(&self) -> bool {
        self.inner.isPowered
    }

    pub fn is_repairing(&self) -> bool {
        self.get_order() == Order::Repair
    }

    pub fn is_researching(&self) -> bool {
        self.get_order() == Order::ResearchTech
    }

    pub fn is_selected(&self) -> bool {
        self.inner.isSelected
    }

    pub fn is_sieged(&self) -> bool {
        matches!(
            self.get_type(),
            UnitType::Terran_Siege_Tank_Siege_Mode | UnitType::Hero_Edmund_Duke_Siege_Mode
        )
    }

    pub fn is_moving(&self) -> bool {
        self.inner.isMoving
    }

    pub fn last_command_frame(&self) -> i32 {
        self.info().last_command_frame
    }

    pub fn is_starting_attack(&self) -> bool {
        self.inner.isStartingAttack
    }

    pub fn is_stasised(&self) -> bool {
        self.inner.stasisTimer != 0
    }

    pub fn is_stimmed(&self) -> bool {
        self.inner.stimTimer != 0
    }

    pub fn is_stuck(&self) -> bool {
        self.inner.isStuck
    }

    pub fn is_targetable(&self) -> bool {
        if !self.exists() {
            return false;
        }
        if !self.is_visible()
            && !self
                .inner
                .game()
                .is_flag_enabled(Flag::CompleteMapInformation)
        {
            return false;
        }

        if self.is_completed()
            && !self.get_type().is_building()
            && !self.is_morphing()
            && self.get_type() != UnitType::Protoss_Archon
            && self.get_type() != UnitType::Protoss_Dark_Archon
        {
            return false;
        }
        matches!(
            self.get_type(),
            UnitType::Spell_Scanner_Sweep
                | UnitType::Spell_Dark_Swarm
                | UnitType::Spell_Disruption_Web
                | UnitType::Special_Map_Revealer
        )
    }

    pub fn is_training(&self) -> bool {
        self.inner.isTraining
    }

    pub fn is_under_attack(&self) -> bool {
        self.inner.recentlyAttacked
    }

    pub fn is_under_dark_swarm(&self) -> bool {
        self.inner.isUnderDarkSwarm
    }

    pub fn is_under_disruption_web(&self) -> bool {
        self.inner.isUnderDWeb
    }

    pub fn is_under_storm(&self) -> bool {
        self.inner.isUnderStorm
    }

    pub fn is_upgrading(&self) -> bool {
        self.get_order() == Order::Upgrade
    }

    pub fn get_id(&self) -> UnitId {
        self.id
    }

    pub fn exists(&self) -> bool {
        self.inner.exists
    }

    pub fn has_path<P: UnitOrPosition>(&self, target: P) -> bool {
        if let Ok(target) = target.to_position() {
            self.is_flying()
                || self.exists()
                    && target.is_valid(&self.inner.game())
                    && (self.inner.game().has_path(self.get_position(), target)
                        || self
                            .inner
                            .game()
                            .has_path((self.get_left(), self.get_top()), target)
                        || self
                            .inner
                            .game()
                            .has_path((self.get_right(), self.get_top()), target)
                        || self
                            .inner
                            .game()
                            .has_path((self.get_left(), self.get_bottom()), target)
                        || self
                            .inner
                            .game()
                            .has_path((self.get_right(), self.get_top()), target))
        } else {
            false
        }
    }

    pub fn is_visible_to(&self, player: &Player) -> bool {
        self.inner.isVisible[player.id]
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible_to(&self.inner.game().self_().unwrap())
    }

    pub fn is_being_constructed(&self) -> bool {
        if self.is_morphing() {
            return true;
        }
        if self.is_completed() {
            return false;
        }
        if self.get_type().get_race() != Race::Terran {
            return true;
        }
        self.get_build_unit().is_some()
    }

    pub fn is_being_gathered(&self) -> bool {
        self.inner.isBeingGathered
    }

    pub fn is_being_healed(&self) -> bool {
        self.get_type().get_race() == Race::Terran
            && self.is_completed()
            && self.get_hit_points() > self.inner.lastHitPoints
    }

    pub fn is_completed(&self) -> bool {
        self.inner.isCompleted
    }

    pub fn is_morphing(&self) -> bool {
        self.inner.isMorphing
    }

    pub fn get_player(&self) -> Player {
        self.inner
            .game()
            .get_player(self.inner.player as usize)
            .unwrap_or_else(|| self.inner.game().neutral())
    }
}

/***
 * Unit Commands
 */
impl Unit {
    pub fn attack<T: UnitOrPosition>(&self, target: T) -> BwResult<bool> {
        let mut cmd = self.command(false);
        target.assign_attack(&mut cmd);
        self.issue_command(cmd)
    }

    pub fn gather(&self, target: &Unit) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            targetIndex: target.id as i32,
            ..self.command_type(UnitCommandType::Gather, false)
        })
    }

    pub fn right_click<T: UnitOrPosition>(&self, target: T) -> BwResult<bool> {
        let mut cmd = self.command(false);
        target.assign_right_click(&mut cmd);
        self.issue_command(cmd)
    }

    pub fn build<P: Into<TilePosition>>(&self, type_: UnitType, target: P) -> BwResult<bool> {
        let target = target.into();
        self.issue_command(UnitCommand {
            x: target.x,
            y: target.y,
            extra: type_ as i32,
            ..self.command_type(UnitCommandType::Build, false)
        })
    }

    pub fn build_addon(&self, type_: UnitType) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            extra: type_ as i32,
            ..self.command_type(UnitCommandType::Build, false)
        })
    }

    pub fn train(&self, type_: UnitType) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            extra: type_ as i32,
            ..self.command_type(UnitCommandType::Train, false)
        })
    }

    pub fn morph(&self, type_: UnitType) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            extra: type_ as i32,
            ..self.command_type(UnitCommandType::Morph, false)
        })
    }

    pub fn research(&self, tech: TechType) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            extra: tech as i32,
            ..self.command_type(UnitCommandType::Research, false)
        })
    }

    pub fn upgrade(&self, upgrade: UpgradeType) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            extra: upgrade as i32,
            ..self.command_type(UnitCommandType::Upgrade, false)
        })
    }

    pub fn set_rally_point<P: UnitOrPosition>(&self, target: P) -> BwResult<bool> {
        let mut cmd = self.command(false);
        target.assign_rally_point(&mut cmd);
        self.issue_command(cmd)
    }

    pub fn move_<P: Into<Position>>(&self, target: P) -> BwResult<bool> {
        let target = target.into();
        self.issue_command(UnitCommand {
            x: target.x,
            y: target.y,
            ..self.command_type(UnitCommandType::Move, false)
        })
    }

    pub fn patrol<P: Into<Position>>(&self, target: P) -> BwResult<bool> {
        let target = target.into();
        self.issue_command(UnitCommand {
            x: target.x,
            y: target.y,
            ..self.command_type(UnitCommandType::Patrol, false)
        })
    }

    pub fn hold_position(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Hold_Position, false)
        })
    }

    pub fn stop(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Stop, false)
        })
    }

    pub fn follow(&self, target: &Unit) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            targetIndex: target.id as i32,
            ..self.command_type(UnitCommandType::Follow, false)
        })
    }

    pub fn return_cargo(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Return_Cargo, false)
        })
    }

    pub fn repair(&self, target: &Unit) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            targetIndex: target.id as i32,
            ..self.command_type(UnitCommandType::Repair, false)
        })
    }

    pub fn burrow(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Burrow, false)
        })
    }

    pub fn unburrow(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Unburrow, false)
        })
    }

    pub fn cloak(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Cloak, false)
        })
    }

    pub fn decloak(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Decloak, false)
        })
    }

    pub fn siege(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Siege, false)
        })
    }

    pub fn unsiege(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Unsiege, false)
        })
    }

    pub fn lift(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Lift, false)
        })
    }

    pub fn land<TP: Into<TilePosition>>(&self, target: TP) -> BwResult<bool> {
        let target = target.into();
        self.issue_command(UnitCommand {
            x: target.x,
            y: target.y,
            ..self.command_type(UnitCommandType::Land, false)
        })
    }

    pub fn load(&self, target: &Unit) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            targetIndex: target.id as i32,
            ..self.command_type(UnitCommandType::Load, false)
        })
    }

    pub fn unload(&self, target: &Unit) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            targetIndex: target.id as i32,
            ..self.command_type(UnitCommandType::Unload, false)
        })
    }

    pub fn unload_all<P: Into<Position>, OP: Into<Option<P>>>(&self, target: OP) -> BwResult<bool> {
        let target = target.into();
        if let Some(target) = target {
            let target = target.into();
            self.issue_command(UnitCommand {
                x: target.x,
                y: target.y,
                ..self.command_type(UnitCommandType::Unload_All, false)
            })
        } else {
            self.issue_command(UnitCommand {
                ..self.command_type(UnitCommandType::Unload_All, false)
            })
        }
    }

    pub fn halt_construction(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Halt_Construction, false)
        })
    }

    pub fn cancel_construction(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Cancel_Construction, false)
        })
    }

    pub fn cancel_addon(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Cancel_Addon, false)
        })
    }

    pub fn cancel_train<S: Into<Option<i32>>>(&self, slot: S) -> BwResult<bool> {
        let slot = slot.into();
        self.issue_command(UnitCommand {
            extra: slot.unwrap_or(-2),
            ..self.command_type(UnitCommandType::Cancel_Train, false)
        })
    }

    pub fn cancel_morph(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Cancel_Morph, false)
        })
    }

    pub fn cancel_research(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Cancel_Research, false)
        })
    }

    pub fn cancel_upgrade(&self) -> BwResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Cancel_Upgrade, false)
        })
    }

    pub fn use_tech<T: UnitOrPosition, OT: Into<Option<T>>>(
        &self,
        tech: TechType,
        target: OT,
    ) -> BwResult<bool> {
        let mut cmd = self.command(false);
        if let Some(target) = target.into() {
            target.assign_use_tech(tech, &mut cmd);
            self.issue_command(cmd)
        } else {
            self.issue_command(UnitCommand {
                extra: tech as i32,
                ..self.command_type(UnitCommandType::Use_Tech, false)
            })
        }
    }

    pub fn place_cop<TP: Into<TilePosition>>(&self, target: TP) -> BwResult<bool> {
        let target = target.into();
        self.issue_command(UnitCommand {
            x: target.x,
            y: target.y,
            ..self.command_type(UnitCommandType::Place_COP, false)
        })
    }

    fn command(&self, shift_queue: bool) -> UnitCommand {
        UnitCommand {
            type_: BWAPI_UnitCommandType {
                _base: UnitCommandType::None as u32,
            },
            extra: shift_queue as i32,
            x: 0,
            y: 0,
            unitIndex: self.id as i32,
            targetIndex: -1,
        }
    }

    fn command_type(&self, cmd: UnitCommandType, shift_queue: bool) -> UnitCommand {
        UnitCommand {
            type_: BWAPI_UnitCommandType { _base: cmd as u32 },
            extra: shift_queue as i32,
            x: 0,
            y: 0,
            unitIndex: self.id as i32,
            targetIndex: -1,
        }
    }

    pub fn issue_command(&self, cmd: UnitCommand) -> BwResult<bool> {
        if !self.can_issue_command(cmd)? {
            return Ok(false);
        }

        let cmd = if (cmd.get_type() == UnitCommandType::Train
            || cmd.get_type() == UnitCommandType::Morph)
            && self.get_type().produces_larva()
            && cmd.get_unit_type().what_builds().0 == UnitType::Zerg_Larva
        {
            let mut larva_cmd: Option<UnitCommand> = None;
            for larva in self.get_larva() {
                if !larva.is_constructing()
                    && larva.is_completed()
                    && larva.can_command().unwrap_or(false)
                {
                    larva_cmd = Some(UnitCommand {
                        unitIndex: larva.get_id() as i32,
                        ..cmd
                    });
                    break;
                }
            }
            if let Some(cmd) = larva_cmd {
                cmd
            } else {
                return Ok(false);
            }
        } else {
            cmd
        };
        self.inner.game().issue_command(cmd);
        let mut info = self.info();
        info.last_command_frame = self.inner.game().get_frame_count();
        self.set_info(info);
        Ok(true)
    }
}

pub enum PathErr {
    UnitNotVisible,
}

pub trait UnitOrPosition {
    fn assign_right_click(&self, cmd: &mut UnitCommand);
    fn assign_attack(&self, cmd: &mut UnitCommand);
    fn assign_rally_point(&self, cmd: &mut UnitCommand);
    fn assign_use_tech(&self, tech: TechType, cmd: &mut UnitCommand);
    fn to_position(&self) -> Result<Position, PathErr>;
}

impl UnitOrPosition for &Unit {
    fn assign_attack(&self, cmd: &mut UnitCommand) {
        cmd.targetIndex = self.id as i32;
        cmd.type_._base = UnitCommandType::Attack_Unit as u32;
    }
    fn assign_right_click(&self, cmd: &mut UnitCommand) {
        cmd.targetIndex = self.id as i32;
        cmd.type_._base = UnitCommandType::Right_Click_Unit as u32;
    }
    fn assign_rally_point(&self, cmd: &mut UnitCommand) {
        cmd.targetIndex = self.id as i32;
        cmd.type_._base = UnitCommandType::Set_Rally_Unit as u32;
    }
    fn assign_use_tech(&self, tech: TechType, cmd: &mut UnitCommand) {
        cmd.targetIndex = self.id as i32;
        cmd.extra = tech as i32;
        cmd.type_._base = UnitCommandType::Use_Tech as u32;
    }
    fn to_position(&self) -> Result<Position, PathErr> {
        if self.exists() {
            Ok(self.get_position())
        } else {
            Err(PathErr::UnitNotVisible)
        }
    }
}

impl<I: Into<Position> + Copy> UnitOrPosition for I {
    fn assign_attack(&self, cmd: &mut UnitCommand) {
        let pos: Position = (*self).into();
        cmd.x = pos.x;
        cmd.y = pos.y;
        cmd.type_._base = UnitCommandType::Attack_Move as u32;
    }
    fn assign_right_click(&self, cmd: &mut UnitCommand) {
        let pos: Position = (*self).into();
        cmd.x = pos.x;
        cmd.y = pos.y;
        cmd.type_._base = UnitCommandType::Right_Click_Position as u32;
    }
    fn assign_rally_point(&self, cmd: &mut UnitCommand) {
        let pos: Position = (*self).into();
        cmd.x = pos.x;
        cmd.y = pos.y;
        cmd.type_._base = UnitCommandType::Set_Rally_Position as u32;
    }
    fn assign_use_tech(&self, tech: TechType, cmd: &mut UnitCommand) {
        let pos: Position = (*self).into();
        cmd.x = pos.x;
        cmd.y = pos.y;
        cmd.extra = tech as i32;
        cmd.type_._base = UnitCommandType::Use_Tech as u32;
    }

    fn to_position(&self) -> Result<Position, PathErr> {
        Ok((*self).into())
    }
}

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
