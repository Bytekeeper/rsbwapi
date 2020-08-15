use crate::unit::Unit;
use crate::force::Force;
use crate::game::Game;
use crate::types::c_str_to_str;
use crate::types::Color;
use bwapi_wrapper::prelude::*;
use bwapi_wrapper::*;
use num_traits::FromPrimitive;

#[derive(Clone, Copy)]
pub struct Player<'a> {
    pub id: usize,
    game: &'a Game<'a>,
    data: &'a BWAPI_PlayerData,
}

impl<'a> Player<'a> {
    pub(crate) fn new(id: usize, game: &'a Game<'a>, data: &'a BWAPI_PlayerData) -> Self {
        Player { id, game, data }
    }

    pub fn all_unit_count(&self, unit: UnitType) -> i32 {
        self.data.allUnitCount[unit as usize]
    }

    pub fn armor(&self, unit: UnitType) -> i32 {
        let mut armor = unit.armor();
        armor += self.get_upgrade_level(unit.armor_upgrade());
        if unit == UnitType::Zerg_Ultralisk
            && self.get_upgrade_level(UpgradeType::Chitinous_Plating) > 0
            || unit == UnitType::Hero_Torrasque
        {
            armor += 2;
        }
        armor
    }

    pub fn completed_unit_count(&self, unit: UnitType) -> i32 {
        self.data.completedUnitCount[unit as usize]
    }

    pub fn damage(&self, wpn: WeaponType) -> i32 {
        let mut dmg = wpn.damage_amount();
        dmg += self.get_upgrade_level(wpn.upgrade_type()) * wpn.damage_bonus();
        dmg * wpn.damage_factor()
    }

    pub fn dead_unit_count(&self, unit: UnitType) -> i32 {
        self.data.deadUnitCount[unit as usize]
    }

    pub fn gas(&self) -> i32 {
        self.data.gas
    }

    pub fn gathered_gas(&self) -> i32 {
        self.data.gatheredGas
    }

    pub fn gathered_minerals(&self) -> i32 {
        self.data.gatheredMinerals
    }

    pub fn get_building_score(&self) -> i32 {
        self.data.totalBuildingScore
    }

    pub fn get_color(&self) -> Color {
        Color::from_i32(self.data.color).unwrap()
    }

    pub fn get_custom_score(&self) -> i32 {
        self.data.customScore
    }

    pub(crate) fn force_id(&self) -> i32 {
        self.data.force
    }

    pub fn get_force(&self) -> Force {
        self.game.get_force(self.force_id())
    }

    pub fn get_id(&self) -> i32 {
        self.id as i32
    }

    pub fn get_kill_score(&self) -> i32 {
        self.data.totalKillScore
    }

    pub fn get_max_upgrade_level(&self, upgrade: UpgradeType) -> i32 {
        self.data.maxUpgradeLevel[upgrade as usize]
    }

    pub fn get_name(&self) -> &str {
        c_str_to_str(&self.data.name)
    }

    pub fn get_race(&self) -> Race {
        Race::new(self.data.race)
    }

    pub fn get_units(&self) -> Vec<Unit> {
        self.game.get_all_units().iter().filter(|u| u.get_player() == Some(*self)).cloned().collect()
    }

    pub fn get_upgrade_level(&self, upgrade_type: UpgradeType) -> i32 {
        self.data.upgradeLevel[upgrade_type as usize]
    }

    pub fn has_researched(&self, tech: TechType) -> bool {
        self.data.hasResearched[tech as usize]
    }

    pub fn has_unit_type_requirement(&self, unit: UnitType, amount: i32) -> bool {
        if unit == UnitType::None {
            return true;
        }

        (match unit {
            UnitType::Zerg_Hatchery => {
                self.completed_unit_count(UnitType::Zerg_Hatchery)
                    + self.all_unit_count(UnitType::Zerg_Lair)
                    + self.all_unit_count(UnitType::Zerg_Hive)
            }
            UnitType::Zerg_Lair => {
                self.completed_unit_count(UnitType::Zerg_Lair)
                    + self.all_unit_count(UnitType::Zerg_Hive)
            }
            UnitType::Zerg_Spire => {
                self.completed_unit_count(UnitType::Zerg_Spire)
                    + self.all_unit_count(UnitType::Zerg_Greater_Spire)
            }
            _ => self.completed_unit_count(unit),
        }) >= amount
    }

    pub fn incomplete_unit_count(&self, unit: UnitType) -> i32 {
        self.all_unit_count(unit) - self.completed_unit_count(unit)
    }

    pub fn is_ally(&self, other: &Player) -> bool {
        self.data.isAlly[other.id]
    }

    pub fn is_defeated(&self) -> bool {
        self.data.isDefeated
    }

    pub fn is_enemy(&self, other: &Player) -> bool {
        self.data.isEnemy[other.id]
    }

    pub fn is_neutral(&self) -> bool {
        self.data.isNeutral
    }

    pub fn is_observer(&self) -> bool {
        !self.data.isParticipating
    }

    pub fn is_research_available(&self, tech: TechType) -> bool {
        self.data.isResearchAvailable[tech as usize]
    }

    pub fn is_reseaching(&self, tech: TechType) -> bool {
        self.data.isResearching[tech as usize]
    }

    pub fn is_unit_available(&self, unit: UnitType) -> bool {
        self.data.isUnitAvailable[unit as usize]
    }

    pub fn is_upgrading(&self, upgrade: UpgradeType) -> bool {
        self.data.isUpgrading[upgrade as usize]
    }

    pub fn is_victorious(&self) -> bool {
        self.data.isVictorious
    }

    pub fn killed_unit_count(&self, unit: UnitType) -> i32 {
        self.data.killedUnitCount[unit as usize]
    }

    pub fn left_game(&self) -> bool {
        self.data.leftGame
    }

    pub fn max_energy(&self, unit: UnitType) -> i32 {
        let mut energy = unit.max_energy();
        if (unit == UnitType::Protoss_Arbiter
            && self.get_upgrade_level(UpgradeType::Khaydarin_Core) > 0)
            || (unit == UnitType::Protoss_Corsair
                && self.get_upgrade_level(UpgradeType::Argus_Jewel) > 0)
            || (unit == UnitType::Protoss_Dark_Archon
                && self.get_upgrade_level(UpgradeType::Argus_Talisman) > 0)
            || (unit == UnitType::Protoss_High_Templar
                && self.get_upgrade_level(UpgradeType::Khaydarin_Amulet) > 0)
            || (unit == UnitType::Terran_Ghost
                && self.get_upgrade_level(UpgradeType::Moebius_Reactor) > 0)
            || (unit == UnitType::Terran_Battlecruiser
                && self.get_upgrade_level(UpgradeType::Colossus_Reactor) > 0)
            || (unit == UnitType::Terran_Science_Vessel
                && self.get_upgrade_level(UpgradeType::Titan_Reactor) > 0)
            || (unit == UnitType::Terran_Wraith
                && self.get_upgrade_level(UpgradeType::Apollo_Reactor) > 0)
            || (unit == UnitType::Terran_Medic
                && self.get_upgrade_level(UpgradeType::Caduceus_Reactor) > 0)
            || (unit == UnitType::Zerg_Defiler
                && self.get_upgrade_level(UpgradeType::Metasynaptic_Node) > 0)
            || (unit == UnitType::Zerg_Queen
                && self.get_upgrade_level(UpgradeType::Gamete_Meiosis) > 0)
        {
            energy += 50
        }
        energy
    }

    pub fn minerals(&self) -> i32 {
        self.data.minerals
    }

    pub fn refunded_gas(&self) -> i32 {
        self.data.refundedGas
    }

    pub fn refunded_minerals(&self) -> i32 {
        self.data.refundedMinerals
    }

    pub fn repaired_gas(&self) -> i32 {
        self.data.repairedGas
    }

    pub fn repaired_minerals(&self) -> i32 {
        self.data.repairedMinerals
    }

    pub fn sight_range(&self, unit: UnitType) -> i32 {
        let mut range = unit.sight_range();
        if (unit == UnitType::Terran_Ghost
            && self.get_upgrade_level(UpgradeType::Ocular_Implants) > 0)
            || (unit == UnitType::Zerg_Overlord
                && self.get_upgrade_level(UpgradeType::Antennae) > 0)
            || (unit == UnitType::Protoss_Observer
                && self.get_upgrade_level(UpgradeType::Sensor_Array) > 0)
            || (unit == UnitType::Protoss_Scout
                && self.get_upgrade_level(UpgradeType::Apial_Sensors) > 0)
        {
            range = 11 * 32
        }
        range
    }

    pub fn spent_gas(&self) -> i32 {
        self.gathered_gas() + self.refunded_gas() - self.gas() - self.repaired_gas()
    }

    pub fn spent_minerals(&self) -> i32 {
        self.gathered_minerals() + self.refunded_minerals()
            - self.minerals()
            - self.repaired_minerals()
    }

    pub fn supply_total(&self) -> i32 {
        self.supply_total_for(self.get_race())
    }

    pub fn supply_total_for(&self, race: Race) -> i32 {
        self.data.supplyTotal[race as usize]
    }

    pub fn supply_used(&self) -> i32 {
        self.supply_used_by(self.get_race())
    }

    pub fn supply_used_by(&self, race: Race) -> i32 {
        self.data.supplyUsed[race as usize]
    }

    pub fn top_speed(&self, unit: UnitType) -> f64 {
        let mut speed = unit.top_speed();
        if (unit == UnitType::Terran_Vulture
            && self.get_upgrade_level(UpgradeType::Ion_Thrusters) > 0)
            || (unit == UnitType::Zerg_Overlord
                && self.get_upgrade_level(UpgradeType::Pneumatized_Carapace) > 0)
            || (unit == UnitType::Zerg_Zergling
                && self.get_upgrade_level(UpgradeType::Metabolic_Boost) > 0)
            || (unit == UnitType::Zerg_Hydralisk
                && self.get_upgrade_level(UpgradeType::Muscular_Augments) > 0)
            || (unit == UnitType::Protoss_Zealot
                && self.get_upgrade_level(UpgradeType::Leg_Enhancements) > 0)
            || (unit == UnitType::Protoss_Shuttle
                && self.get_upgrade_level(UpgradeType::Gravitic_Drive) > 0)
            || (unit == UnitType::Protoss_Observer
                && self.get_upgrade_level(UpgradeType::Gravitic_Boosters) > 0)
            || (unit == UnitType::Protoss_Scout
                && self.get_upgrade_level(UpgradeType::Gravitic_Thrusters) > 0)
            || (unit == UnitType::Zerg_Ultralisk
                && self.get_upgrade_level(UpgradeType::Anabolic_Synthesis) > 0)
        {
            if unit == UnitType::Protoss_Scout {
                speed += 427.0 / 256.0;
            } else {
                speed *= 1.5;
            }
            if speed < 853.0 / 256.0 {
                speed = 853.0 / 256.0;
            }
        }
        speed
    }

    pub fn visible_unit_count(&self, unit: UnitType) -> i32 {
        self.data.visibleUnitCount[unit as usize]
    }

    pub fn weapon_damage_cooldown(&self, unit: UnitType) -> i32 {
        let mut cooldown = unit.ground_weapon().damage_cooldown();
        if unit == UnitType::Zerg_Zergling
            && self.get_upgrade_level(UpgradeType::Adrenal_Glands) > 0
        {
            // Divide cooldown by 2
            cooldown /= 2;
            // Prevent cooldown from going out of bounds
            cooldown = cooldown.max(5).min(250);
        }
        cooldown
    }

    pub fn weapon_max_range(&self, weapon: WeaponType) -> i32 {
        let mut range = weapon.max_range();
        if (weapon == WeaponType::Gauss_Rifle
            && self.get_upgrade_level(UpgradeType::U_238_Shells) > 0)
            || (weapon == WeaponType::Needle_Spines
                && self.get_upgrade_level(UpgradeType::Grooved_Spines) > 0)
        {
            range += 32; // 1 *
        } else if weapon == WeaponType::Phase_Disruptor
            && self.get_upgrade_level(UpgradeType::Singularity_Charge) > 0
        {
            range += 2 * 32;
        } else if weapon == WeaponType::Hellfire_Missile_Pack
            && self.get_upgrade_level(UpgradeType::Charon_Boosters) > 0
        {
            range += 3 * 32;
        }
        range
    }
}

impl<'a> PartialEq for Player<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
