use crate::player::Player;

use crate::*;
use bwapi_wrapper::*;

#[derive(Clone, Copy)]
pub struct Unit<'a> {
    pub id: usize,
    frame: &'a Frame<'a>,
    data: &'a BWAPI_UnitData,
}

impl<'a> Unit<'a> {
    pub(crate) fn new(id: usize, frame: &'a Frame<'a>, data: &'a BWAPI_UnitData) -> Self {
        Unit { id, frame, data }
    }

    pub fn get_closest_unit(&self, pred: impl Fn(&Unit) -> bool) -> Option<&Unit> {
        self.frame.get_all_units().iter().find(|u| pred(u))
    }

    pub fn get_type(&self) -> UnitType {
        types::unit_type_from(self.data.type_)
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
        types::unit_type_from(self.data.buildType)
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
        unimplemented!()
    }

    pub fn get_interceptor_count(&self) -> i32 {
        self.data.interceptorCount
    }

    pub fn get_interceptors(&self) -> Vec<&Unit<'a>> {
        self.frame
            .get_all_units()
            .iter()
            .filter(|&u| {
                if let Some(carrier) = u.get_carrier() {
                    carrier == *self
                } else {
                    false
                }
            })
            .collect()
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

    pub fn get_spell_cooldown(&self) -> i32 {
        self.data.spellCooldown
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
