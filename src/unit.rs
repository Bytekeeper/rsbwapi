use crate::player::Player;

use crate::*;
use bwapi_wrapper::*;
use can_do::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct UnitInfo {
    pub id: usize,
    pub initial_hit_points: i32,
    pub initial_resources: i32,
    pub initial_position: Position,
    pub initial_type: UnitType,
}

impl UnitInfo {
    pub(crate) fn new(id: usize, data: &BWAPI_UnitData) -> Self {
        Self {
            id,
            initial_hit_points: data.hitPoints,
            initial_resources: data.resources,
            initial_position: Position {
                x: data.positionX,
                y: data.positionY,
            },
            initial_type: UnitType::new(data.type_),
        }
    }
}

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

#[derive(Clone, Copy)]
pub struct Unit<'a> {
    pub id: usize,
    pub(crate) game: &'a Game<'a>,
    data: &'a BWAPI_UnitData,
    info: UnitInfo,
}

impl<'a> Unit<'a> {
    pub(crate) fn new(
        id: usize,
        game: &'a Game<'a>,
        data: &'a BWAPI_UnitData,
        info: UnitInfo,
    ) -> Self {
        Unit {
            id,
            game,
            data,
            info,
        }
    }

    pub(crate) fn get_buttonset(&self) -> i32 {
        self.data.buttonset
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

    pub fn get_closest_unit(&self, pred: impl Fn(&Unit) -> bool) -> Option<&Unit> {
        self.game.get_all_units().iter().find(|u| pred(u))
    }

    pub fn get_type(&self) -> UnitType {
        UnitType::new(self.data.type_)
    }

    pub fn is_accelerating(&self) -> bool {
        self.data.isAccelerating
    }

    pub fn is_attacking(&self) -> bool {
        self.data.isAttacking
    }

    pub fn is_attack_game(&self) -> bool {
        self.data.isAttackFrame
    }

    pub fn get_acid_spore_count(&self) -> i32 {
        self.data.acidSporeCount
    }

    pub fn get_addon(&self) -> Option<Unit> {
        self.game.get_unit(self.data.addon)
    }

    pub fn get_air_weapon_cooldown(&self) -> i32 {
        self.data.airWeaponCooldown
    }

    pub fn get_angle(&self) -> f64 {
        self.data.angle
    }

    pub fn get_bottom(&self) -> i32 {
        self.get_position().y + self.get_type().dimension_down()
    }

    pub fn get_build_type(&self) -> UnitType {
        UnitType::new(self.data.buildType)
    }

    pub fn get_build_unit(&self) -> Option<Unit> {
        self.game.get_unit(self.data.buildUnit)
    }

    pub fn get_carrier(&self) -> Option<Unit> {
        self.game.get_unit(self.data.carrier)
    }

    pub fn get_defense_matrix_points(&self) -> i32 {
        self.data.defenseMatrixPoints
    }

    pub fn get_defense_matrix_timer(&self) -> i32 {
        self.data.defenseMatrixTimer
    }

    pub fn get_distance(&self, target: &Unit) -> i32 {
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
        self.data.energy
    }

    pub fn get_ensnare_timer(&self) -> i32 {
        self.data.ensnareTimer
    }

    pub fn get_ground_weapon_cooldown(&self) -> i32 {
        self.data.groundWeaponCooldown
    }

    pub fn get_hatchery(&self) -> Option<Unit> {
        self.game.get_unit(self.data.hatchery)
    }

    pub fn get_hit_points(&self) -> i32 {
        self.data.hitPoints
    }

    pub fn get_initial_hit_points(&self) -> i32 {
        self.info.initial_hit_points
    }

    pub fn get_initial_resources(&self) -> i32 {
        self.info.initial_resources
    }

    pub fn get_initial_tile_position(&self) -> TilePosition {
        (self.info.initial_position - self.info.initial_type.tile_size().to_position() / 2)
            .to_tile_position()
    }

    pub fn get_interceptor_count(&self) -> i32 {
        self.data.interceptorCount
    }

    pub fn get_interceptors(&self) -> Vec<Unit<'a>> {
        if self.get_type() != UnitType::Protoss_Carrier
            && self.get_type() != UnitType::Hero_Gantrithor
        {
            return vec![];
        }
        let burrowed_map = self.game.connected_units.borrow();
        let interceptors = burrowed_map.get(&self.id);
        if let Some(interceptors) = interceptors {
            return interceptors
                .iter()
                .map(|&i| self.game.get_unit(i).expect("Interceptor to be present"))
                .collect();
        }
        let interceptors: Vec<Unit> = self
            .game
            .get_all_units()
            .iter()
            .filter(|u| u.get_carrier() == Some(*self))
            .cloned()
            .collect();
        self.game
            .connected_units
            .borrow_mut()
            .insert(self.id, interceptors.iter().map(|u| u.id as i32).collect());
        interceptors
    }

    pub fn get_irradiate_timer(&self) -> i32 {
        self.data.irradiateTimer
    }

    pub fn get_kill_count(&self) -> i32 {
        self.data.killCount
    }

    pub fn get_larva(&self) -> Vec<Unit> {
        if !self.get_type().produces_larva() {
            return vec![];
        }
        if let Some(larva) = self.game.connected_units.borrow().get(&self.id) {
            return larva
                .iter()
                .map(|&i| self.game.get_unit(i).expect("Larva to be present"))
                .collect();
        }
        let larva: Vec<Unit> = self
            .game
            .get_all_units()
            .iter()
            .filter(|u| u.get_hatchery() == Some(*self))
            .cloned()
            .collect();
        self.game
            .connected_units
            .borrow_mut()
            .insert(self.id, larva.iter().map(|u| u.id as i32).collect());
        larva
    }

    pub fn get_last_attacking_player(&self) -> Option<Player> {
        self.game.get_player(self.data.lastAttackerPlayer)
    }

    pub fn get_left(&self) -> i32 {
        self.get_position().x - self.get_type().dimension_left()
    }

    pub fn get_loaded_units(&self) -> Vec<Unit> {
        let map = self.game.loaded_units.borrow();
        let loaded_units = map.get(&self.id);
        if let Some(loaded_units) = loaded_units {
            loaded_units
                .iter()
                .map(|&i| self.game.get_unit(i).expect("Loaded unit to be present"))
                .collect()
        } else {
            let loaded_units: Vec<Unit> = self
                .game
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
                .collect();
            self.game
                .loaded_units
                .borrow_mut()
                .insert(self.id, loaded_units.iter().map(|u| u.id as i32).collect());
            loaded_units
        }
    }

    pub fn get_lockdown_timer(&self) -> i32 {
        self.data.lockdownTimer
    }

    pub fn get_maelstrom_timer(&self) -> i32 {
        self.data.maelstromTimer
    }

    pub fn get_nydus_exit(&self) -> Option<Unit> {
        self.game.get_unit(self.data.nydusExit)
    }

    pub fn get_order(&self) -> Order {
        Order::new(self.data.order)
    }

    pub fn get_order_target(&self) -> Option<Unit> {
        self.game.get_unit(self.data.orderTarget)
    }

    pub fn get_order_target_position(&self) -> Option<Position> {
        Position::new(
            self.data.orderTargetPositionX,
            self.data.orderTargetPositionY,
        )
    }

    pub fn get_plague_timer(&self) -> i32 {
        self.data.plagueTimer
    }

    pub fn get_position(&self) -> Position {
        Position {
            x: self.data.positionX,
            y: self.data.positionY,
        }
    }

    pub fn get_power_up(&self) -> Option<Unit> {
        self.game.get_unit(self.data.powerUp)
    }

    pub fn get_rally_position(&self) -> Option<Position> {
        if self.data.rallyPositionX < 0 {
            None
        } else {
            Some(Position {
                x: self.data.rallyPositionX,
                y: self.data.rallyPositionY,
            })
        }
    }

    pub fn get_rally_unit(&self) -> Option<Unit> {
        self.game.get_unit(self.data.rallyUnit)
    }

    pub fn get_remaining_build_time(&self) -> i32 {
        self.data.remainingBuildTime
    }

    pub fn get_remaining_research_time(&self) -> i32 {
        self.data.remainingResearchTime
    }

    pub fn get_remaining_train_time(&self) -> i32 {
        self.data.remainingTrainTime
    }

    pub fn get_remove_timer(&self) -> i32 {
        self.data.removeTimer
    }

    pub fn get_replay_id(&self) -> i32 {
        self.data.replayID
    }

    pub fn get_resource_group(&self) -> i32 {
        self.data.resourceGroup
    }

    pub fn get_resources(&self) -> i32 {
        self.data.resources
    }

    pub fn get_right(&self) -> i32 {
        self.get_position().x + self.get_type().dimension_right()
    }

    pub fn get_scarab_count(&self) -> i32 {
        self.data.scarabCount
    }

    pub fn get_secondary_order(&self) -> Order {
        Order::new(self.data.secondaryOrder)
    }

    pub fn get_shields(&self) -> i32 {
        self.data.shields
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
        self.data.spellCooldown
    }

    pub fn get_spider_mine_count(&self) -> i32 {
        self.data.spiderMineCount
    }

    pub fn get_stasis_timer(&self) -> i32 {
        self.data.stasisTimer
    }

    pub fn get_stim_timer(&self) -> i32 {
        self.data.stimTimer
    }

    pub fn get_target(&self) -> Option<Unit> {
        self.game.get_unit(self.id as i32)
    }

    pub fn get_target_position(&self) -> Option<Position> {
        Position::new(self.data.targetPositionX, self.data.targetPositionY)
    }

    pub fn get_tech(&self) -> TechType {
        TechType::new(self.data.tech)
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
        (0..self.data.trainingQueueCount as usize)
            .map(|i| self.data.trainingQueue[i])
            .map(UnitType::new)
            .collect()
    }

    pub fn get_transport(&self) -> Option<Unit> {
        self.game.get_unit(self.data.transport as i32)
    }

    pub fn get_upgrade(&self) -> UpgradeType {
        UpgradeType::new(self.data.upgrade)
    }

    pub fn get_velocity(&self) -> Vector2D {
        Vector2D {
            x: self.data.velocityX,
            y: self.data.velocityY,
        }
    }

    pub fn has_nuke(&self) -> bool {
        self.data.hasNuke
    }

    pub fn is_blind(&self) -> bool {
        self.data.isBlind
    }

    pub fn is_braking(&self) -> bool {
        self.data.isBraking
    }

    pub fn is_burrowed(&self) -> bool {
        self.data.isBurrowed
    }

    pub fn is_carrying_gas(&self) -> bool {
        self.data.carryResourceType == 1
    }

    pub fn is_carrying_minerals(&self) -> bool {
        self.data.carryResourceType == 2
    }

    pub fn is_constructing(&self) -> bool {
        self.data.isConstructing
    }

    pub fn is_defense_matrixed(&self) -> bool {
        self.data.defenseMatrixTimer > 0
    }

    pub fn is_detected(&self) -> bool {
        self.data.isDetected
    }

    pub fn is_ensnared(&self) -> bool {
        self.data.ensnareTimer > 0
    }

    pub fn is_flying(&self) -> bool {
        self.get_type().is_flyer() || self.is_lifted()
    }

    pub fn is_following(&self) -> bool {
        self.get_order() == Order::Follow
    }

    pub fn is_gathering_gas(&self) -> bool {
        if !self.data.isGathering {
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
            return self.data.carryResourceType == 1;
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
        if !self.data.isGathering {
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
            return self.data.carryResourceType == 2;
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
        self.data.isHallucination
    }

    pub fn is_holding_position(&self) -> bool {
        self.get_order() == Order::HoldPosition
    }

    pub fn is_idle(&self) -> bool {
        self.data.isIdle
    }

    pub fn is_interruptible(&self) -> bool {
        self.data.isInterruptible
    }

    pub fn is_invincible(&self) -> bool {
        self.data.isInvincible
    }

    pub fn is_in_weapon_range(&self, target: &Unit) -> bool {
        if !self.exists() || !target.exists() || self == target {
            return false;
        }

        let this_type = self.get_type();

        let wpn = if target.is_flying() {
            this_type.air_weapon()
        } else {
            this_type.ground_weapon()
        };

        if wpn == WeaponType::None || wpn == WeaponType::Unknown {
            return false;
        }

        let min_range = wpn.min_range();
        let max_range = wpn.max_range();
        let distance = self.get_distance(target);
        (min_range == 0 || min_range < distance) && distance <= max_range
    }

    pub fn is_irradiated(&self) -> bool {
        self.data.irradiateTimer != 0
    }

    pub fn is_lifted(&self) -> bool {
        self.data.isLifted
    }

    pub fn is_loaded(&self) -> bool {
        self.get_transport().is_some()
    }

    pub fn is_locked_down(&self) -> bool {
        self.data.lockdownTimer != 0
    }

    pub fn is_maelstrommed(&self) -> bool {
        self.data.maelstromTimer != 0
    }

    pub fn is_parasited(&self) -> bool {
        self.data.isParasited
    }

    pub fn is_patrolling(&self) -> bool {
        self.get_order() == Order::Patrol
    }

    pub fn is_plagued(&self) -> bool {
        self.data.plagueTimer != 0
    }

    pub fn is_powered(&self) -> bool {
        self.data.isPowered
    }

    pub fn is_repairing(&self) -> bool {
        self.get_order() == Order::Repair
    }

    pub fn is_researching(&self) -> bool {
        self.get_order() == Order::ResearchTech
    }

    pub fn is_selected(&self) -> bool {
        self.data.isSelected
    }

    pub fn is_sieged(&self) -> bool {
        match self.get_type() {
            UnitType::Terran_Siege_Tank_Siege_Mode | UnitType::Hero_Edmund_Duke_Siege_Mode => true,
            _ => false,
        }
    }

    pub fn is_starting_attack(&self) -> bool {
        self.data.isStartingAttack
    }

    pub fn is_stasised(&self) -> bool {
        self.data.stasisTimer != 0
    }

    pub fn is_stimmed(&self) -> bool {
        self.data.stimTimer != 0
    }

    pub fn is_stuck(&self) -> bool {
        self.data.isStuck
    }

    pub fn is_targetable(&self) -> bool {
        if !self.exists() {
            return false;
        }
        if !self.is_visible() && !self.game.is_flag_enabled(Flag::CompleteMapInformation) {
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
        match self.get_type() {
            UnitType::Spell_Scanner_Sweep
            | UnitType::Spell_Dark_Swarm
            | UnitType::Spell_Disruption_Web
            | UnitType::Special_Map_Revealer => false,
            _ => true,
        }
    }

    pub fn is_training(&self) -> bool {
        self.data.isTraining
    }

    pub fn is_under_attack(&self) -> bool {
        self.data.recentlyAttacked
    }

    pub fn is_under_dark_swarm(&self) -> bool {
        self.data.isUnderDarkSwarm
    }

    pub fn is_under_disruption_web(&self) -> bool {
        self.data.isUnderDWeb
    }

    pub fn is_under_storm(&self) -> bool {
        self.data.isUnderStorm
    }

    pub fn is_upgrading(&self) -> bool {
        self.get_order() == Order::Upgrade
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn exists(&self) -> bool {
        self.data.exists
    }

    pub fn has_path<P: UnitOrPosition>(&self, target: P) -> bool {
        if let Ok(target) = target.to_position() {
            self.is_flying()
                || self.exists()
                    && target.is_valid()
                    && (self.game.has_path(self.get_position(), target)
                        || self
                            .game
                            .has_path((self.get_left(), self.get_top()), target)
                        || self
                            .game
                            .has_path((self.get_right(), self.get_top()), target)
                        || self
                            .game
                            .has_path((self.get_left(), self.get_bottom()), target)
                        || self
                            .game
                            .has_path((self.get_right(), self.get_top()), target))
        } else {
            false
        }
    }

    pub fn is_visible_to(&self, player: &Player) -> bool {
        self.data.isVisible[player.id as usize]
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible_to(&self.game.self_().unwrap())
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
        self.get_build_unit() != None
    }

    pub fn is_being_gathered(&self) -> bool {
        self.data.isBeingGathered
    }

    pub fn is_being_healed(&self) -> bool {
        self.get_type().get_race() == Race::Terran
            && self.is_completed()
            && self.get_hit_points() > self.data.lastHitPoints
    }

    pub fn is_completed(&self) -> bool {
        self.data.isCompleted
    }

    pub fn is_morphing(&self) -> bool {
        self.data.isMorphing
    }

    pub fn get_player(&self) -> Player {
        self.game
            .get_player(self.data.player)
            .unwrap_or(self.game.neutral())
    }
}

/***
 * Unit Commands
 */
impl<'a> Unit<'a> {
    pub fn attack<T: UnitOrPosition>(&self, target: &T) -> BWResult<bool> {
        let mut cmd = self.command(false);
        target.assign_attack(&mut cmd);
        self.issue_command(cmd)
    }

    pub fn gather(&self, target: &Unit) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            targetIndex: target.id as i32,
            ..self.command_type(UnitCommandType::Gather, false)
        })
    }

    pub fn right_click<T: UnitOrPosition>(&self, target: &T) -> BWResult<bool> {
        let mut cmd = self.command(false);
        target.assign_right_click(&mut cmd);
        self.issue_command(cmd)
    }

    pub fn build<P: Into<TilePosition>>(&self, type_: UnitType, target: P) -> BWResult<bool> {
        let target = target.into();
        self.issue_command(UnitCommand {
            x: target.x,
            y: target.y,
            extra: type_ as i32,
            ..self.command_type(UnitCommandType::Build, false)
        })
    }

    pub fn build_addon(&self, type_: UnitType) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            extra: type_ as i32,
            ..self.command_type(UnitCommandType::Build, false)
        })
    }

    pub fn train(&self, type_: UnitType) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            extra: type_ as i32,
            ..self.command_type(UnitCommandType::Train, false)
        })
    }

    pub fn morph(&self, type_: UnitType) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            extra: type_ as i32,
            ..self.command_type(UnitCommandType::Morph, false)
        })
    }

    pub fn research(&self, tech: TechType) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            extra: tech as i32,
            ..self.command_type(UnitCommandType::Research, false)
        })
    }

    pub fn upgrade(&self, upgrade: UpgradeType) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            extra: upgrade as i32,
            ..self.command_type(UnitCommandType::Upgrade, false)
        })
    }

    pub fn set_rally_point<P: UnitOrPosition>(&self, target: P) -> BWResult<bool> {
        let mut cmd = self.command(false);
        target.assign_rally_point(&mut cmd);
        self.issue_command(cmd)
    }

    pub fn move_<P: Into<Position>>(&self, target: P) -> BWResult<bool> {
        let target = target.into();
        self.issue_command(UnitCommand {
            x: target.x,
            y: target.y,
            ..self.command_type(UnitCommandType::Move, false)
        })
    }

    pub fn patrol<P: Into<Position>>(&self, target: P) -> BWResult<bool> {
        let target = target.into();
        self.issue_command(UnitCommand {
            x: target.x,
            y: target.y,
            ..self.command_type(UnitCommandType::Patrol, false)
        })
    }

    pub fn hold_position(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Hold_Position, false)
        })
    }

    pub fn stop(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Stop, false)
        })
    }

    pub fn follow(&self, target: &Unit) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            targetIndex: target.id as i32,
            ..self.command_type(UnitCommandType::Follow, false)
        })
    }

    pub fn return_cargo(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Return_Cargo, false)
        })
    }

    pub fn repair(&self, target: &Unit) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            targetIndex: target.id as i32,
            ..self.command_type(UnitCommandType::Repair, false)
        })
    }

    pub fn burrow(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Burrow, false)
        })
    }

    pub fn unburrow(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Unburrow, false)
        })
    }

    pub fn cloak(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Cloak, false)
        })
    }

    pub fn decloak(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Decloak, false)
        })
    }

    pub fn siege(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Siege, false)
        })
    }

    pub fn unsiege(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Unsiege, false)
        })
    }

    pub fn lift(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Lift, false)
        })
    }

    pub fn land<TP: Into<TilePosition>>(&self, target: TP) -> BWResult<bool> {
        let target = target.into();
        self.issue_command(UnitCommand {
            x: target.x,
            y: target.y,
            ..self.command_type(UnitCommandType::Land, false)
        })
    }

    pub fn load(&self, target: &Unit) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            targetIndex: target.id as i32,
            ..self.command_type(UnitCommandType::Load, false)
        })
    }

    pub fn unload(&self, target: &Unit) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            targetIndex: target.id as i32,
            ..self.command_type(UnitCommandType::Unload, false)
        })
    }

    pub fn unload_all<P: Into<Position>, OP: Into<Option<P>>>(&self, target: OP) -> BWResult<bool> {
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

    pub fn halt_construction(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Halt_Construction, false)
        })
    }

    pub fn cancel_construction(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Cancel_Construction, false)
        })
    }

    pub fn cancel_addon(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Cancel_Addon, false)
        })
    }

    pub fn cancel_train<S: Into<Option<i32>>>(&self, slot: S) -> BWResult<bool> {
        let slot = slot.into();
        self.issue_command(UnitCommand {
            extra: slot.unwrap_or(-2),
            ..self.command_type(UnitCommandType::Cancel_Train, false)
        })
    }

    pub fn cancel_morph(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Cancel_Morph, false)
        })
    }

    pub fn cancel_research(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Cancel_Research, false)
        })
    }

    pub fn cancel_upgrade(&self) -> BWResult<bool> {
        self.issue_command(UnitCommand {
            ..self.command_type(UnitCommandType::Cancel_Upgrade, false)
        })
    }

    pub fn use_tech<T: UnitOrPosition, OT: Into<Option<T>>>(
        &self,
        tech: TechType,
        target: OT,
    ) -> BWResult<bool> {
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

    pub fn place_cop<TP: Into<TilePosition>>(&self, target: TP) -> BWResult<bool> {
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
                .get_unit(arg.c.targetIndex)
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
    ) -> BWResult<bool> {
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

    pub fn can_set_rally_position(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_move(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_patrol(&self, check_commandability: bool) -> BWResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.can_move(false)? {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn can_return_cargo(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_hold_position(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_stop(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_burrow(&self, check_commandability: bool) -> BWResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.can_use_tech_without_target(TechType::Burrowing, true, false)? {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn can_unburrow(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_cloak(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_decloak(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_siege(&self, check_commandability: bool) -> BWResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.can_use_tech_without_target(TechType::Tank_Siege_Mode, true, false)? {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn can_unsiege(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_lift(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_unload_with_or_without_target(&self, check_commandability: bool) -> BWResult<bool> {
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
    ) -> BWResult<bool> {
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

    pub fn can_unload_all(&self, check_commandability: bool) -> BWResult<bool> {
        self.can_unload_at_position(self.get_position(), true, check_commandability)
    }

    pub fn can_right_click(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_right_click_position(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_halt_construction(&self, check_commandability: bool) -> BWResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if self.get_order() != Order::ConstructingBuilding {
            return Err(Error::Incompatible_State);
        }

        Ok(true)
    }

    pub fn can_cancel_construction(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_cancel_addon(&self, check_commandability: bool) -> BWResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.get_addon().map(|a| a.is_completed()).unwrap_or(false) {
            return Err(Error::Incompatible_UnitType);
        }
        Ok(true)
    }

    pub fn can_cancel_research(&self, check_commandability: bool) -> BWResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if self.get_order() == Order::ResearchTech {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_cancel_upgrade(&self, check_commandability: bool) -> BWResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if self.get_order() != Order::Upgrade {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_cancel_morph(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn issue_command(&self, cmd: UnitCommand) -> BWResult<bool> {
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
        self.game.issue_command(cmd);
        Ok(true)
    }

    pub fn can_build(&self, checker: impl CanBuild) -> BWResult<bool> {
        checker.can_build(self)
    }

    pub fn can_build_addon(&self, checker: impl CanBuildAddon) -> BWResult<bool> {
        checker.can_build_addon(self)
    }

    pub fn can_train(&self, checker: impl CanTrain) -> BWResult<bool> {
        checker.can_train(self)
    }

    pub fn can_morph(&self, checker: impl CanMorph) -> BWResult<bool> {
        checker.can_morph(self)
    }

    pub fn can_attack(&self, checker: impl CanAttack) -> BWResult<bool> {
        checker.can_attack(self)
    }

    pub fn can_attack_unit(&self, checker: impl CanAttackUnit) -> BWResult<bool> {
        checker.can_attack_unit(self)
    }

    pub fn can_attack_move(&self, check_commandability: bool) -> BWResult<bool> {
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

    pub fn can_cancel_train(&self, check_commandability: bool) -> BWResult<bool> {
        if check_commandability && !self.can_command()? {
            return Ok(false);
        }

        if !self.is_training() {
            return Err(Error::Incompatible_State);
        }
        Ok(true)
    }

    pub fn can_cancel_train_slot(&self, checker: impl CanCancelTrainSlot) -> BWResult<bool> {
        checker.can_cancel_train_slot(self)
    }

    pub fn can_follow(&self, checker: impl CanFollow) -> BWResult<bool> {
        checker.can_follow(self)
    }

    pub fn can_land(&self, checker: impl CanLand) -> BWResult<bool> {
        checker.can_land(self)
    }

    pub fn can_load(&self, checker: impl CanLoad) -> BWResult<bool> {
        checker.can_load(self)
    }

    pub fn can_gather(&self, checker: impl CanGather) -> BWResult<bool> {
        checker.can_gather(self)
    }

    pub fn can_place_cop(&self, checker: impl CanPlaceCop) -> BWResult<bool> {
        checker.can_place_cop(self)
    }

    pub fn can_repair(&self, checker: impl CanRepair) -> BWResult<bool> {
        checker.can_repair(self)
    }

    pub fn can_right_click_unit(&self, checker: impl CanRightClickUnit) -> BWResult<bool> {
        checker.can_right_click_unit(self)
    }

    pub fn can_set_rally_unit(&self, checker: impl CanSetRallyUnit) -> BWResult<bool> {
        checker.can_set_rally_unit(self)
    }

    pub fn can_unload(&self, checker: impl CanUnload) -> BWResult<bool> {
        checker.can_unload(self)
    }

    pub fn can_unload_all_position(&self, checker: impl CanUnloadAllPosition) -> BWResult<bool> {
        checker.can_unload_all_position(self)
    }

    pub fn can_use_tech(&self, checker: impl CanUseTech) -> BWResult<bool> {
        checker.can_use_tech(self)
    }

    pub fn can_use_tech_position(&self, checker: impl CanUseTechPosition) -> BWResult<bool> {
        checker.can_use_tech_position(self)
    }

    pub fn can_use_tech_unit(&self, checker: impl CanUseTechUnit) -> BWResult<bool> {
        checker.can_use_tech_unit(self)
    }

    pub fn can_use_tech_with_or_without_target(
        &self,
        checker: impl CanUseTechWithOrWithoutTarget,
    ) -> BWResult<bool> {
        checker.can_use_tech_with_or_without_target(self)
    }

    pub fn can_use_tech_without_target(
        &self,
        tech: TechType,
        check_can_issue_command_type: bool,
        check_commandability: bool,
    ) -> BWResult<bool> {
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

impl UnitOrPosition for &Unit<'_> {
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

impl UnitOrPosition for Position {
    fn assign_attack(&self, cmd: &mut UnitCommand) {
        cmd.x = self.x;
        cmd.y = self.y;
        cmd.type_._base = UnitCommandType::Attack_Move as u32;
    }
    fn assign_right_click(&self, cmd: &mut UnitCommand) {
        cmd.x = self.x;
        cmd.y = self.y;
        cmd.type_._base = UnitCommandType::Right_Click_Position as u32;
    }
    fn assign_rally_point(&self, cmd: &mut UnitCommand) {
        cmd.x = self.x;
        cmd.y = self.y;
        cmd.type_._base = UnitCommandType::Set_Rally_Position as u32;
    }
    fn assign_use_tech(&self, tech: TechType, cmd: &mut UnitCommand) {
        cmd.x = self.x;
        cmd.y = self.y;
        cmd.extra = tech as i32;
        cmd.type_._base = UnitCommandType::Use_Tech as u32;
    }

    fn to_position(&self) -> Result<Position, PathErr> {
        Ok(*self)
    }
}

impl<'a> PartialEq for Unit<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

trait UnitCommandExt {
    fn get_target(&self) -> Option<Unit>;
}
