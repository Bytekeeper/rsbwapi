use crate::player::Player;

use crate::*;
use bwapi_wrapper::*;

#[derive(Clone, Copy)]
pub struct Unit<'a> {
    pub id: usize,
    game: &'a Game<'a>,
    data: &'a BWAPI_UnitData,
    info: UnitInfo,
}

#[derive(Clone, Copy)]
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
        let burrowed_map = self.game.interceptors.borrow();
        let interceptors = burrowed_map.get(&self.id);
        if let Some(interceptors) = interceptors {
            interceptors
                .iter()
                .map(|&i| self.game.get_unit(i).expect("Interceptor to be present"))
                .collect()
        } else {
            let interceptors: Vec<Unit> = self
                .game
                .get_all_units()
                .iter()
                .filter(|u| {
                    if let Some(carrier) = u.get_carrier() {
                        carrier == *self
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();
            self.game
                .interceptors
                .borrow_mut()
                .insert(self.id, interceptors.iter().map(|u| u.id as i32).collect());
            interceptors
        }
    }

    pub fn get_irradiate_timer(&self) -> i32 {
        self.data.irradiateTimer
    }

    pub fn get_kill_count(&self) -> i32 {
        self.data.killCount
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

    pub fn get_plague_timer(&self) -> i32 {
        self.data.plagueTimer
    }

    pub fn get_position(&self) -> Position {
        Position {
            x: self.data.positionX,
            y: self.data.positionY,
        }
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

    pub fn is_flying(&self) -> bool {
        self.get_type().is_flyer() || self.is_lifted()
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
        unimplemented!()
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

    pub fn get_player(&self) -> Option<Player> {
        self.game.get_player(self.data.player)
    }

    pub fn attack<T: UnitOrPosition>(&self, target: &T) {
        let mut cmd = self.command(false);
        target.assign_attack(&mut cmd);
        self.issue_command(cmd);
    }

    pub fn gather(&self, target: &Unit) {
        self.issue_command(UnitCommand {
            targetIndex: target.id as i32,
            ..self.command_type(UnitCommandType::Gather, false)
        })
    }

    pub fn right_click<T: UnitOrPosition>(&self, target: T) {
        let mut cmd = self.command(false);
        target.assign_right_click(&mut cmd);
        self.issue_command(cmd);
    }

    pub fn build<P: Into<TilePosition>>(&self, type_: UnitType, target: P) {
        let target = target.into();
        self.issue_command(UnitCommand {
            x: target.x,
            y: target.y,
            extra: type_ as i32,
            ..self.command_type(UnitCommandType::Build, false)
        });
    }

    pub fn build_addon(&self, type_: UnitType) {
        self.issue_command(UnitCommand {
            extra: type_ as i32,
            ..self.command_type(UnitCommandType::Build, false)
        });
    }

    pub fn train(&self, type_: UnitType) {
        self.issue_command(UnitCommand {
            extra: type_ as i32,
            ..self.command_type(UnitCommandType::Train, false)
        });
    }

    pub fn morph(&self, type_: UnitType) {
        self.issue_command(UnitCommand {
            extra: type_ as i32,
            ..self.command_type(UnitCommandType::Morph, false)
        });
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

    pub fn issue_command(&self, cmd: UnitCommand) {
        self.game.cmd.borrow_mut().issue_command(cmd)
    }
}

pub trait UnitOrPosition {
    fn assign_right_click(&self, cmd: &mut UnitCommand);
    fn assign_attack(&self, cmd: &mut UnitCommand);
}

impl UnitOrPosition for Unit<'_> {
    fn assign_attack(&self, cmd: &mut UnitCommand) {
        cmd.targetIndex = self.id as i32;
        cmd.type_._base = UnitCommandType::Attack_Unit as u32;
    }
    fn assign_right_click(&self, cmd: &mut UnitCommand) {
        cmd.targetIndex = self.id as i32;
        cmd.type_._base = UnitCommandType::Right_Click_Unit as u32;
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
}

impl<'a> PartialEq for Unit<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub type UnitCommand = BWAPIC_UnitCommand;
