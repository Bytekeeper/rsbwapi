use crate::player::Player;

use crate::types::{Race, TechType, TypeFrom, UnitTypeExt};
use crate::*;
use bwapi_wrapper::*;

#[derive(Clone, Copy)]
pub struct Unit<'a> {
    pub id: usize,
    frame: &'a Frame<'a>,
    data: &'a BWAPI_UnitData,
    info: UnitInfo,
}

#[derive(Clone, Copy)]
pub(crate) struct UnitInfo {
    pub id: usize,
    pub initial_hit_points: i32,
    pub initial_resources: i32,
}

impl UnitInfo {
    pub(crate) fn new(id: usize, data: &BWAPI_UnitData) -> Self {
        Self {
            id,
            initial_hit_points: data.hitPoints,
            initial_resources: data.resources,
        }
    }
}

impl<'a> Unit<'a> {
    pub(crate) fn new(
        id: usize,
        frame: &'a Frame<'a>,
        data: &'a BWAPI_UnitData,
        info: UnitInfo,
    ) -> Self {
        Unit {
            id,
            frame,
            data,
            info,
        }
    }

    pub fn get_closest_unit(&self, pred: impl Fn(&Unit) -> bool) -> Option<&Unit> {
        self.frame.get_all_units().iter().find(|u| pred(u))
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

    pub fn is_attack_frame(&self) -> bool {
        self.data.isAttackFrame
    }

    pub fn get_acid_spore_count(&self) -> i32 {
        self.data.acidSporeCount
    }

    pub fn get_addon(&self) -> Option<Unit> {
        self.frame.get_unit(self.data.addon)
    }

    pub fn get_air_weapon_cooldown(&self) -> i32 {
        self.data.airWeaponCooldown
    }

    pub fn get_angle(&self) -> f64 {
        self.data.angle
    }

    pub fn get_build_type(&self) -> UnitType {
        UnitType::new(self.data.buildType)
    }

    pub fn get_build_unit(&self) -> Option<Unit> {
        self.frame.get_unit(self.data.buildUnit)
    }

    pub fn get_carrier(&self) -> Option<Unit> {
        self.frame.get_unit(self.data.carrier)
    }

    pub fn get_defense_matrix_points(&self) -> i32 {
        self.data.defenseMatrixPoints
    }

    pub fn get_defense_matrix_timer(&self) -> i32 {
        self.data.defenseMatrixTimer
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
        self.frame.get_unit(self.data.hatchery)
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

    pub fn get_interceptor_count(&self) -> i32 {
        self.data.interceptorCount
    }

    pub fn get_interceptors(&self) -> Vec<Unit<'a>> {
        let burrowed_map = self.frame.interceptors.borrow();
        let interceptors = burrowed_map.get(&self.id);
        if let Some(interceptors) = interceptors {
            interceptors
                .iter()
                .map(|&i| self.frame.get_unit(i).expect("Interceptor to be present"))
                .collect()
        } else {
            let interceptors: Vec<Unit> = self
                .frame
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
            self.frame
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
        self.frame.get_player(self.data.lastAttackerPlayer)
    }

    pub fn get_loaded_units(&self) -> Vec<Unit> {
        let map = self.frame.loaded_units.borrow();
        let loaded_units = map.get(&self.id);
        if let Some(loaded_units) = loaded_units {
            loaded_units
                .iter()
                .map(|&i| self.frame.get_unit(i).expect("Loaded unit to be present"))
                .collect()
        } else {
            let loaded_units: Vec<Unit> = self
                .frame
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
            self.frame
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
        self.frame.get_unit(self.data.nydusExit)
    }

    pub fn get_order_target(&self) -> Option<Unit> {
        self.frame.get_unit(self.data.orderTarget)
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
        self.frame.get_unit(self.data.rallyUnit)
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
        self.frame.get_unit(self.id as i32)
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

    pub fn get_transport(&self) -> Option<Unit> {
        self.frame.get_unit(self.id as i32)
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn exists(&self) -> bool {
        self.data.exists
    }

    pub fn is_visible(&self, player: &Player) -> bool {
        self.data.isVisible[player.id as usize]
    }

    pub fn is_being_constructed(&self) -> bool {
        if self.is_morphing() {
            return true;
        }
        if self.is_completed() {
            return false;
        }
        if self.get_type().get_race() != types::Race::Terran {
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
        self.frame.get_player(self.data.player)
    }

    pub fn gather(&self, target: &Unit) -> UnitCommand {
        UnitCommand {
            targetIndex: target.id as i32,
            ..self.command_type(BWAPI_UnitCommandTypes_Enum_Enum::Gather)
        }
    }
    pub fn attack(&self, target: &Unit) -> UnitCommand {
        UnitCommand {
            targetIndex: target.id as i32,
            ..self.command_type(BWAPI_UnitCommandTypes_Enum_Enum::Attack_Unit)
        }
    }

    fn command_type(&self, cmd: BWAPI_UnitCommandTypes_Enum_Enum) -> UnitCommand {
        UnitCommand {
            type_: BWAPI_UnitCommandType { _base: cmd as u32 },
            extra: 0,
            x: 0,
            y: 0,
            unitIndex: self.id as i32,
            targetIndex: -1,
        }
    }
}

impl<'a> PartialEq for Unit<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub type UnitCommand = BWAPIC_UnitCommand;
