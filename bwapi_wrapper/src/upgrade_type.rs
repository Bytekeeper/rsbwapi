use crate::prelude::*;
pub(crate) struct UpgradeTypeData {
    pub(crate) gas_price_factor: i32,
    pub(crate) max_repeats: i32,
    pub(crate) what_upgrades: UnitType,
    pub(crate) what_uses: &'static [UnitType],
    pub(crate) whats_required: UnitType,
    pub(crate) upgrade_time: i32,
    pub(crate) mineral_price_factor: i32,
    pub(crate) upgrade_time_factor: i32,
    pub(crate) race: Race,
    pub(crate) mineral_price: i32,
    pub(crate) gas_price: i32,
    pub(crate) name: &'static str,
}
pub(crate) static UPGRADE_TYPE_DATA: [UpgradeTypeData; 53] = [
    UpgradeTypeData {
        gas_price_factor: 75,
        max_repeats: 3,
        what_upgrades: UnitType::Terran_Engineering_Bay,
        what_uses: &[
            UnitType::Terran_Marine,
            UnitType::Terran_Ghost,
            UnitType::Terran_SCV,
            UnitType::Hero_Gui_Montag,
            UnitType::Terran_Civilian,
            UnitType::Hero_Sarah_Kerrigan,
            UnitType::Hero_Jim_Raynor_Marine,
            UnitType::Terran_Firebat,
            UnitType::Terran_Medic,
            UnitType::Hero_Samir_Duran,
            UnitType::Hero_Alexei_Stukov,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 75,
        upgrade_time_factor: 480,
        race: Race::Terran,
        mineral_price: 100,
        gas_price: 100,
        name: "Terran_Infantry_Armor",
    },
    UpgradeTypeData {
        gas_price_factor: 75,
        max_repeats: 3,
        what_upgrades: UnitType::Terran_Armory,
        what_uses: &[
            UnitType::Terran_Vulture,
            UnitType::Terran_Goliath,
            UnitType::Terran_Siege_Tank_Tank_Mode,
            UnitType::Hero_Alan_Schezar,
            UnitType::Hero_Jim_Raynor_Vulture,
            UnitType::Hero_Edmund_Duke_Tank_Mode,
            UnitType::Hero_Edmund_Duke_Siege_Mode,
            UnitType::Terran_Siege_Tank_Siege_Mode,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 75,
        upgrade_time_factor: 480,
        race: Race::Terran,
        mineral_price: 100,
        gas_price: 100,
        name: "Terran_Vehicle_Plating",
    },
    UpgradeTypeData {
        gas_price_factor: 75,
        max_repeats: 3,
        what_upgrades: UnitType::Terran_Armory,
        what_uses: &[
            UnitType::Terran_Wraith,
            UnitType::Terran_Science_Vessel,
            UnitType::Terran_Dropship,
            UnitType::Terran_Battlecruiser,
            UnitType::Hero_Tom_Kazansky,
            UnitType::Hero_Magellan,
            UnitType::Hero_Arcturus_Mengsk,
            UnitType::Hero_Hyperion,
            UnitType::Hero_Norad_II,
            UnitType::Terran_Valkyrie,
            UnitType::Hero_Gerard_DuGalle,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 75,
        upgrade_time_factor: 480,
        race: Race::Terran,
        mineral_price: 150,
        gas_price: 150,
        name: "Terran_Ship_Plating",
    },
    UpgradeTypeData {
        gas_price_factor: 75,
        max_repeats: 3,
        what_upgrades: UnitType::Zerg_Evolution_Chamber,
        what_uses: &[
            UnitType::Zerg_Larva,
            UnitType::Zerg_Egg,
            UnitType::Zerg_Zergling,
            UnitType::Zerg_Hydralisk,
            UnitType::Zerg_Ultralisk,
            UnitType::Zerg_Broodling,
            UnitType::Zerg_Drone,
            UnitType::Zerg_Defiler,
            UnitType::Hero_Torrasque,
            UnitType::Zerg_Infested_Terran,
            UnitType::Hero_Infested_Kerrigan,
            UnitType::Hero_Unclean_One,
            UnitType::Hero_Hunter_Killer,
            UnitType::Hero_Devouring_One,
            UnitType::Zerg_Cocoon,
            UnitType::Zerg_Lurker_Egg,
            UnitType::Zerg_Lurker,
            UnitType::Hero_Infested_Duran,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 75,
        upgrade_time_factor: 480,
        race: Race::Zerg,
        mineral_price: 150,
        gas_price: 150,
        name: "Zerg_Carapace",
    },
    UpgradeTypeData {
        gas_price_factor: 75,
        max_repeats: 3,
        what_upgrades: UnitType::Zerg_Spire,
        what_uses: &[
            UnitType::Zerg_Overlord,
            UnitType::Zerg_Mutalisk,
            UnitType::Zerg_Guardian,
            UnitType::Zerg_Queen,
            UnitType::Zerg_Scourge,
            UnitType::Hero_Matriarch,
            UnitType::Hero_Kukulza_Mutalisk,
            UnitType::Hero_Kukulza_Guardian,
            UnitType::Hero_Yggdrasill,
            UnitType::Zerg_Devourer,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 75,
        upgrade_time_factor: 480,
        race: Race::Zerg,
        mineral_price: 150,
        gas_price: 150,
        name: "Zerg_Flyer_Carapace",
    },
    UpgradeTypeData {
        gas_price_factor: 75,
        max_repeats: 3,
        what_upgrades: UnitType::Protoss_Forge,
        what_uses: &[
            UnitType::Protoss_Dark_Templar,
            UnitType::Protoss_Dark_Archon,
            UnitType::Protoss_Probe,
            UnitType::Protoss_Zealot,
            UnitType::Protoss_Dragoon,
            UnitType::Protoss_High_Templar,
            UnitType::Protoss_Archon,
            UnitType::Hero_Dark_Templar,
            UnitType::Hero_Zeratul,
            UnitType::Hero_Tassadar_Zeratul_Archon,
            UnitType::Hero_Fenix_Zealot,
            UnitType::Hero_Fenix_Dragoon,
            UnitType::Hero_Tassadar,
            UnitType::Hero_Warbringer,
            UnitType::Protoss_Reaver,
            UnitType::Hero_Aldaris,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 75,
        upgrade_time_factor: 480,
        race: Race::Protoss,
        mineral_price: 100,
        gas_price: 100,
        name: "Protoss_Ground_Armor",
    },
    UpgradeTypeData {
        gas_price_factor: 75,
        max_repeats: 3,
        what_upgrades: UnitType::Protoss_Cybernetics_Core,
        what_uses: &[
            UnitType::Protoss_Corsair,
            UnitType::Protoss_Shuttle,
            UnitType::Protoss_Scout,
            UnitType::Protoss_Arbiter,
            UnitType::Protoss_Carrier,
            UnitType::Protoss_Interceptor,
            UnitType::Hero_Mojo,
            UnitType::Hero_Gantrithor,
            UnitType::Protoss_Observer,
            UnitType::Hero_Danimoth,
            UnitType::Hero_Artanis,
            UnitType::Hero_Raszagal,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 75,
        upgrade_time_factor: 480,
        race: Race::Protoss,
        mineral_price: 150,
        gas_price: 150,
        name: "Protoss_Air_Armor",
    },
    UpgradeTypeData {
        gas_price_factor: 75,
        max_repeats: 3,
        what_upgrades: UnitType::Terran_Engineering_Bay,
        what_uses: &[
            UnitType::Terran_Marine,
            UnitType::Hero_Jim_Raynor_Marine,
            UnitType::Terran_Ghost,
            UnitType::Hero_Sarah_Kerrigan,
            UnitType::Terran_Firebat,
            UnitType::Hero_Gui_Montag,
            UnitType::Special_Wall_Flame_Trap,
            UnitType::Special_Right_Wall_Flame_Trap,
            UnitType::Hero_Samir_Duran,
            UnitType::Hero_Alexei_Stukov,
            UnitType::Hero_Infested_Duran,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 75,
        upgrade_time_factor: 480,
        race: Race::Terran,
        mineral_price: 100,
        gas_price: 100,
        name: "Terran_Infantry_Weapons",
    },
    UpgradeTypeData {
        gas_price_factor: 75,
        max_repeats: 3,
        what_upgrades: UnitType::Terran_Armory,
        what_uses: &[
            UnitType::Terran_Vulture,
            UnitType::Hero_Jim_Raynor_Vulture,
            UnitType::Terran_Goliath,
            UnitType::Hero_Alan_Schezar,
            UnitType::Terran_Siege_Tank_Tank_Mode,
            UnitType::Terran_Siege_Tank_Siege_Mode,
            UnitType::Hero_Edmund_Duke_Tank_Mode,
            UnitType::Hero_Edmund_Duke_Siege_Mode,
            UnitType::Special_Floor_Missile_Trap,
            UnitType::Special_Floor_Gun_Trap,
            UnitType::Special_Wall_Missile_Trap,
            UnitType::Special_Right_Wall_Missile_Trap,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 75,
        upgrade_time_factor: 480,
        race: Race::Terran,
        mineral_price: 100,
        gas_price: 100,
        name: "Terran_Vehicle_Weapons",
    },
    UpgradeTypeData {
        gas_price_factor: 50,
        max_repeats: 3,
        what_upgrades: UnitType::Terran_Armory,
        what_uses: &[
            UnitType::Terran_Wraith,
            UnitType::Hero_Tom_Kazansky,
            UnitType::Terran_Battlecruiser,
            UnitType::Hero_Hyperion,
            UnitType::Hero_Norad_II,
            UnitType::Hero_Arcturus_Mengsk,
            UnitType::Hero_Gerard_DuGalle,
            UnitType::Terran_Valkyrie,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 50,
        upgrade_time_factor: 480,
        race: Race::Terran,
        mineral_price: 100,
        gas_price: 100,
        name: "Terran_Ship_Weapons",
    },
    UpgradeTypeData {
        gas_price_factor: 50,
        max_repeats: 3,
        what_upgrades: UnitType::Zerg_Evolution_Chamber,
        what_uses: &[
            UnitType::Zerg_Zergling,
            UnitType::Hero_Devouring_One,
            UnitType::Hero_Infested_Kerrigan,
            UnitType::Zerg_Ultralisk,
            UnitType::Hero_Torrasque,
            UnitType::Zerg_Broodling,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 50,
        upgrade_time_factor: 480,
        race: Race::Zerg,
        mineral_price: 100,
        gas_price: 100,
        name: "Zerg_Melee_Attacks",
    },
    UpgradeTypeData {
        gas_price_factor: 50,
        max_repeats: 3,
        what_upgrades: UnitType::Zerg_Evolution_Chamber,
        what_uses: &[
            UnitType::Zerg_Hydralisk,
            UnitType::Hero_Hunter_Killer,
            UnitType::Zerg_Lurker,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 50,
        upgrade_time_factor: 480,
        race: Race::Zerg,
        mineral_price: 100,
        gas_price: 100,
        name: "Zerg_Missile_Attacks",
    },
    UpgradeTypeData {
        gas_price_factor: 75,
        max_repeats: 3,
        what_upgrades: UnitType::Zerg_Spire,
        what_uses: &[
            UnitType::Zerg_Mutalisk,
            UnitType::Hero_Kukulza_Mutalisk,
            UnitType::Hero_Kukulza_Guardian,
            UnitType::Zerg_Guardian,
            UnitType::Zerg_Devourer,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 75,
        upgrade_time_factor: 480,
        race: Race::Zerg,
        mineral_price: 100,
        gas_price: 100,
        name: "Zerg_Flyer_Attacks",
    },
    UpgradeTypeData {
        gas_price_factor: 50,
        max_repeats: 3,
        what_upgrades: UnitType::Protoss_Forge,
        what_uses: &[
            UnitType::Protoss_Zealot,
            UnitType::Hero_Fenix_Zealot,
            UnitType::Protoss_Dragoon,
            UnitType::Hero_Fenix_Dragoon,
            UnitType::Hero_Tassadar,
            UnitType::Hero_Aldaris,
            UnitType::Protoss_Archon,
            UnitType::Hero_Tassadar_Zeratul_Archon,
            UnitType::Hero_Dark_Templar,
            UnitType::Hero_Zeratul,
            UnitType::Protoss_Dark_Templar,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 50,
        upgrade_time_factor: 480,
        race: Race::Protoss,
        mineral_price: 100,
        gas_price: 100,
        name: "Protoss_Ground_Weapons",
    },
    UpgradeTypeData {
        gas_price_factor: 75,
        max_repeats: 3,
        what_upgrades: UnitType::Protoss_Cybernetics_Core,
        what_uses: &[
            UnitType::Protoss_Scout,
            UnitType::Hero_Mojo,
            UnitType::Protoss_Arbiter,
            UnitType::Hero_Danimoth,
            UnitType::Protoss_Interceptor,
            UnitType::Protoss_Carrier,
            UnitType::Protoss_Corsair,
            UnitType::Hero_Artanis,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 75,
        upgrade_time_factor: 480,
        race: Race::Protoss,
        mineral_price: 100,
        gas_price: 100,
        name: "Protoss_Air_Weapons",
    },
    UpgradeTypeData {
        gas_price_factor: 100,
        max_repeats: 3,
        what_upgrades: UnitType::Protoss_Forge,
        what_uses: &[
            UnitType::Protoss_Corsair,
            UnitType::Protoss_Dark_Templar,
            UnitType::Protoss_Dark_Archon,
            UnitType::Protoss_Probe,
            UnitType::Protoss_Zealot,
            UnitType::Protoss_Dragoon,
            UnitType::Protoss_High_Templar,
            UnitType::Protoss_Archon,
            UnitType::Protoss_Shuttle,
            UnitType::Protoss_Scout,
            UnitType::Protoss_Arbiter,
            UnitType::Protoss_Carrier,
            UnitType::Protoss_Interceptor,
            UnitType::Hero_Dark_Templar,
            UnitType::Hero_Zeratul,
            UnitType::Hero_Tassadar_Zeratul_Archon,
            UnitType::Hero_Fenix_Zealot,
            UnitType::Hero_Fenix_Dragoon,
            UnitType::Hero_Tassadar,
            UnitType::Hero_Mojo,
            UnitType::Hero_Warbringer,
            UnitType::Hero_Gantrithor,
            UnitType::Protoss_Reaver,
            UnitType::Protoss_Observer,
            UnitType::Hero_Danimoth,
            UnitType::Hero_Aldaris,
            UnitType::Hero_Artanis,
            UnitType::Hero_Raszagal,
        ],
        whats_required: UnitType::None,
        upgrade_time: 4000,
        mineral_price_factor: 100,
        upgrade_time_factor: 480,
        race: Race::Protoss,
        mineral_price: 200,
        gas_price: 200,
        name: "Protoss_Plasma_Shields",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Terran_Academy,
        what_uses: &[UnitType::Terran_Marine],
        whats_required: UnitType::None,
        upgrade_time: 1500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Terran,
        mineral_price: 150,
        gas_price: 150,
        name: "U_238_Shells",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Terran_Machine_Shop,
        what_uses: &[UnitType::Terran_Vulture],
        whats_required: UnitType::None,
        upgrade_time: 1500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Terran,
        mineral_price: 100,
        gas_price: 100,
        name: "Ion_Thrusters",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Terran_Science_Facility,
        what_uses: &[UnitType::Terran_Science_Vessel],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Terran,
        mineral_price: 150,
        gas_price: 150,
        name: "Titan_Reactor",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Terran_Covert_Ops,
        what_uses: &[UnitType::Terran_Ghost],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Terran,
        mineral_price: 100,
        gas_price: 100,
        name: "Ocular_Implants",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Terran_Covert_Ops,
        what_uses: &[UnitType::Terran_Ghost],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Terran,
        mineral_price: 150,
        gas_price: 150,
        name: "Moebius_Reactor",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Terran_Control_Tower,
        what_uses: &[UnitType::Terran_Wraith],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Terran,
        mineral_price: 200,
        gas_price: 200,
        name: "Apollo_Reactor",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Terran_Physics_Lab,
        what_uses: &[UnitType::Terran_Battlecruiser],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Terran,
        mineral_price: 150,
        gas_price: 150,
        name: "Colossus_Reactor",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Zerg_Lair,
        what_uses: &[UnitType::Zerg_Overlord],
        whats_required: UnitType::None,
        upgrade_time: 2400,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Zerg,
        mineral_price: 200,
        gas_price: 200,
        name: "Ventral_Sacs",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Zerg_Lair,
        what_uses: &[UnitType::Zerg_Overlord],
        whats_required: UnitType::None,
        upgrade_time: 2000,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Zerg,
        mineral_price: 150,
        gas_price: 150,
        name: "Antennae",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Zerg_Lair,
        what_uses: &[UnitType::Zerg_Overlord],
        whats_required: UnitType::None,
        upgrade_time: 2000,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Zerg,
        mineral_price: 150,
        gas_price: 150,
        name: "Pneumatized_Carapace",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Zerg_Spawning_Pool,
        what_uses: &[UnitType::Zerg_Zergling],
        whats_required: UnitType::None,
        upgrade_time: 1500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Zerg,
        mineral_price: 100,
        gas_price: 100,
        name: "Metabolic_Boost",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Zerg_Spawning_Pool,
        what_uses: &[UnitType::Zerg_Zergling],
        whats_required: UnitType::Zerg_Hive,
        upgrade_time: 1500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Zerg,
        mineral_price: 200,
        gas_price: 200,
        name: "Adrenal_Glands",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Zerg_Hydralisk_Den,
        what_uses: &[UnitType::Zerg_Hydralisk],
        whats_required: UnitType::None,
        upgrade_time: 1500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Zerg,
        mineral_price: 150,
        gas_price: 150,
        name: "Muscular_Augments",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Zerg_Hydralisk_Den,
        what_uses: &[UnitType::Zerg_Hydralisk],
        whats_required: UnitType::None,
        upgrade_time: 1500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Zerg,
        mineral_price: 150,
        gas_price: 150,
        name: "Grooved_Spines",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Zerg_Queens_Nest,
        what_uses: &[UnitType::Zerg_Queen],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Zerg,
        mineral_price: 150,
        gas_price: 150,
        name: "Gamete_Meiosis",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Zerg_Defiler_Mound,
        what_uses: &[UnitType::Zerg_Defiler],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Zerg,
        mineral_price: 150,
        gas_price: 150,
        name: "Metasynaptic_Node",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Cybernetics_Core,
        what_uses: &[UnitType::Protoss_Dragoon],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 150,
        gas_price: 150,
        name: "Singularity_Charge",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Citadel_of_Adun,
        what_uses: &[UnitType::Protoss_Zealot],
        whats_required: UnitType::None,
        upgrade_time: 2000,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 150,
        gas_price: 150,
        name: "Leg_Enhancements",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Robotics_Support_Bay,
        what_uses: &[UnitType::Protoss_Reaver],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 200,
        gas_price: 200,
        name: "Scarab_Damage",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Robotics_Support_Bay,
        what_uses: &[UnitType::Protoss_Reaver],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 200,
        gas_price: 200,
        name: "Reaver_Capacity",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Robotics_Support_Bay,
        what_uses: &[UnitType::Protoss_Shuttle],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 200,
        gas_price: 200,
        name: "Gravitic_Drive",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Observatory,
        what_uses: &[UnitType::Protoss_Observer],
        whats_required: UnitType::None,
        upgrade_time: 2000,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 150,
        gas_price: 150,
        name: "Sensor_Array",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Observatory,
        what_uses: &[UnitType::Protoss_Observer],
        whats_required: UnitType::None,
        upgrade_time: 2000,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 150,
        gas_price: 150,
        name: "Gravitic_Boosters",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Templar_Archives,
        what_uses: &[UnitType::Protoss_High_Templar],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 150,
        gas_price: 150,
        name: "Khaydarin_Amulet",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Fleet_Beacon,
        what_uses: &[UnitType::Protoss_Scout],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 100,
        gas_price: 100,
        name: "Apial_Sensors",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Fleet_Beacon,
        what_uses: &[UnitType::Protoss_Scout],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 200,
        gas_price: 200,
        name: "Gravitic_Thrusters",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Fleet_Beacon,
        what_uses: &[UnitType::Protoss_Carrier],
        whats_required: UnitType::None,
        upgrade_time: 1500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 100,
        gas_price: 100,
        name: "Carrier_Capacity",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Arbiter_Tribunal,
        what_uses: &[UnitType::Protoss_Arbiter],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 150,
        gas_price: 150,
        name: "Khaydarin_Core",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Fleet_Beacon,
        what_uses: &[UnitType::Protoss_Corsair],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 100,
        gas_price: 100,
        name: "Argus_Jewel",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Protoss_Templar_Archives,
        what_uses: &[UnitType::Protoss_Dark_Archon],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Protoss,
        mineral_price: 150,
        gas_price: 150,
        name: "Argus_Talisman",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Terran_Academy,
        what_uses: &[UnitType::Terran_Medic],
        whats_required: UnitType::None,
        upgrade_time: 2500,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Terran,
        mineral_price: 150,
        gas_price: 150,
        name: "Caduceus_Reactor",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Zerg_Ultralisk_Cavern,
        what_uses: &[UnitType::Zerg_Ultralisk],
        whats_required: UnitType::None,
        upgrade_time: 2000,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Zerg,
        mineral_price: 150,
        gas_price: 150,
        name: "Chitinous_Plating",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Zerg_Ultralisk_Cavern,
        what_uses: &[UnitType::Zerg_Ultralisk],
        whats_required: UnitType::None,
        upgrade_time: 2000,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Zerg,
        mineral_price: 200,
        gas_price: 200,
        name: "Anabolic_Synthesis",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 1,
        what_upgrades: UnitType::Terran_Machine_Shop,
        what_uses: &[UnitType::Terran_Goliath],
        whats_required: UnitType::Terran_Armory,
        upgrade_time: 2000,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Terran,
        mineral_price: 100,
        gas_price: 100,
        name: "Charon_Boosters",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 0,
        what_upgrades: UnitType::None,
        what_uses: &[
            UnitType::Terran_Vulture_Spider_Mine,
            UnitType::Critter_Ursadon,
            UnitType::Critter_Scantid,
            UnitType::Critter_Rhynadon,
            UnitType::Critter_Ragnasaur,
            UnitType::Critter_Kakaru,
            UnitType::Critter_Bengalaas,
            UnitType::Special_Cargo_Ship,
            UnitType::Special_Mercenary_Gunship,
            UnitType::Terran_SCV,
            UnitType::Protoss_Probe,
            UnitType::Zerg_Drone,
            UnitType::Zerg_Infested_Terran,
            UnitType::Zerg_Scourge,
        ],
        whats_required: UnitType::None,
        upgrade_time: 0,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::None,
        mineral_price: 0,
        gas_price: 0,
        name: "Upgrade_60",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 0,
        what_upgrades: UnitType::None,
        what_uses: &[],
        whats_required: UnitType::None,
        upgrade_time: 0,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::None,
        mineral_price: 0,
        gas_price: 0,
        name: "None",
    },
    UpgradeTypeData {
        gas_price_factor: 0,
        max_repeats: 0,
        what_upgrades: UnitType::None,
        what_uses: &[],
        whats_required: UnitType::None,
        upgrade_time: 0,
        mineral_price_factor: 0,
        upgrade_time_factor: 0,
        race: Race::Unknown,
        mineral_price: 0,
        gas_price: 0,
        name: "Unknown",
    },
];
impl UpgradeType {
    fn d(&self) -> &UpgradeTypeData {
        &UPGRADE_TYPE_DATA[*self as usize]
    }
    pub fn gas_price_factor(&self) -> i32 {
        self.d().gas_price_factor
    }
    pub fn max_repeats(&self) -> i32 {
        self.d().max_repeats
    }
    pub fn what_upgrades(&self) -> UnitType {
        self.d().what_upgrades
    }
    pub fn what_uses(&self) -> &'static [UnitType] {
        self.d().what_uses
    }
    pub fn whats_required(&self) -> UnitType {
        self.d().whats_required
    }
    pub fn upgrade_time(&self) -> i32 {
        self.d().upgrade_time
    }
    pub fn mineral_price_factor(&self) -> i32 {
        self.d().mineral_price_factor
    }
    pub fn upgrade_time_factor(&self) -> i32 {
        self.d().upgrade_time_factor
    }
    pub fn get_race(&self) -> Race {
        self.d().race
    }
    pub fn mineral_price(&self) -> i32 {
        self.d().mineral_price
    }
    pub fn gas_price(&self) -> i32 {
        self.d().gas_price
    }
    pub fn name(&self) -> &'static str {
        self.d().name
    }
}
