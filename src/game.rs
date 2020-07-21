use crate::aimodule::AiModule;
use crate::bullet::Bullet;
use crate::command::Commands;
use crate::position::*;
use crate::shm::Shm;
use crate::types::c_str_to_str;
use crate::unit::UnitInfo;
use crate::*;
use bwapi_wrapper::*;
use core::cell::RefCell;
use std::collections::HashMap;

use crate::player::Player;
use crate::unit::Unit;

pub struct Game {
    data: Shm<BWAPI_GameData>,
    unit_infos: [Option<UnitInfo>; 10000],
    visible_units: Vec<i32>
}

pub struct Frame<'a> {
    data: &'a BWAPI_GameData,
    units: Vec<Unit<'a>>,
    infos: &'a [Option<UnitInfo>; 10000],
    pub(crate) interceptors: RefCell<HashMap<usize, Vec<i32>>>,
}

impl<'a> Frame<'a> {
    pub fn get_player(&self, i: i32) -> Option<Player> {
        if !(0..self.data.playerCount).contains(&i) {
            None
        } else {
            let i = i as usize;
            let data = self.data.players.get(i)?;
            Some(Player::new(i, &data))
        }
    }

    pub fn get_unit(&self, id: i32) -> Option<Unit> {
        if !(0..10000_i32).contains(&id) {
            None
        } else {
            let id = id as usize;
            Some(Unit::new(
                id,
                self,
                &self.data.units[id],
                self.infos[id].expect("UnitInfo to exist"),
            ))
        }
    }
    pub fn get_players(&self) -> Vec<Player> {
        (0..self.data.playerCount as usize)
            .map(|i| Player::new(i, &self.data.players[i as usize]))
            .collect()
    }

    pub fn get_selected_units(&self) -> Vec<Unit> {
        (0..self.data.selectedUnitCount as usize)
            .map(|i| self.data.selectedUnits[i])
            .map(|i| self.get_unit(i).expect("Selected unit to exist"))
            .collect()
    }

    pub fn is_multiplayer(&self) -> bool {
        self.data.isMultiplayer
    }

    pub fn is_walkable<P: Into<WalkPosition>>(&self, wp: P) -> bool {
        let p = wp.into();
        self.data.isWalkable[p.y as usize][p.x as usize]
    }

    pub fn is_visible<P: Into<TilePosition>>(&self, tp: P) -> bool {
        let p = tp.into();
        self.data.isVisible[p.y as usize][p.x as usize]
    }

    pub fn is_buildable<P: Into<TilePosition>>(&self, tp: P) -> bool {
        let p = tp.into();
        self.data.isBuildable[p.y as usize][p.x as usize]
    }

    pub fn is_explored<P: Into<TilePosition>>(&self, tp: P) -> bool {
        let p = tp.into();
        self.data.isExplored[p.y as usize][p.x as usize]
    }

    pub fn has_creep<P: Into<TilePosition>>(&self, tp: P) -> bool {
        let p = tp.into();
        self.data.hasCreep[p.y as usize][p.x as usize]
    }

    pub fn get_ground_height<P: Into<TilePosition>>(&self, tp: P) -> i32 {
        let p = tp.into();
        self.data.getGroundHeight[p.y as usize][p.x as usize]
    }

    pub fn map_height(&self) -> i32 {
        self.data.mapHeight
    }

    pub fn map_width(&self) -> i32 {
        self.data.mapWidth
    }

    pub fn map_hash(&self) -> &str {
        c_str_to_str(&self.data.mapHash)
    }

    pub fn map_file_name(&self) -> &str {
        c_str_to_str(&self.data.mapFileName)
    }

    pub fn map_path_name(&self) -> &str {
        c_str_to_str(&self.data.mapPathName)
    }

    pub fn map_name(&self) -> &str {
        c_str_to_str(&self.data.mapName)
    }

    pub fn get_all_units(&self) -> &Vec<Unit<'a>> {
        &self.units
    }

    pub fn get_allies(&self) -> Vec<Player> {
        let self_ = self.self_();
        if let Some(self_) = self_ {
            self.get_players()
                .iter()
                .filter(|p| p.is_ally(&self_))
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }

    pub fn get_bullets(&self) -> Vec<Bullet> {
        self.data
            .bullets
            .iter()
            .map(|b| Bullet::new(b.id as usize, self, b))
            .filter(|b| b.exists())
            .collect()
    }

    pub fn get_geysers(&self) -> Vec<Unit<'a>> {
        self.get_all_units()
            .iter()
            .filter(|u| u.get_type() == UnitType::Resource_Vespene_Geyser)
            .cloned()
            .collect()
    }

    pub fn enemy(&self) -> Option<Player> {
        self.get_player(self.data.enemy)
    }

    pub fn self_(&self) -> Option<Player> {
        self.get_player(self.data.self_)
    }

    pub fn get_frame_count(&self) -> i32 {
        self.data.frameCount
    }

    pub fn get_nuke_dots(&self) -> Vec<Position> {
        (0..self.data.nukeDotCount as usize)
            .map(|i| self.data.nukeDots[i])
            .map(|p| Position { x: p.x, y: p.y })
            .collect()
    }

    pub fn get_start_locations(&self) -> Vec<TilePosition> {
        (0..self.data.startLocationCount as usize)
            .map(|i| self.data.startLocations[i])
            .map(|p| TilePosition { x: p.x, y: p.y })
            .collect()
    }

    fn event_str(&self, i: usize) -> &str {
        c_str_to_str(&self.data.eventStrings[i])
    }
}

impl Game {
    pub(crate) fn new(data: Shm<BWAPI_GameData>) -> Self {
        Game {
            data,
            unit_infos: [None; 10000],
            visible_units: vec![]
        }
    }

    pub fn is_in_game(&self) -> bool {
        self.data.get().isInGame
    }

    fn with_frame(&self, cb: impl FnOnce(&Frame)) {
        let mut frame = Frame {
            data: self.data.get(),
            units: vec![],
            infos: &self.unit_infos,
            interceptors: RefCell::new(HashMap::new()),
        };
        let unmoved_frame = &frame as *const Frame;
        // SAFETY: Only the infos will be modified here and only a reference of Frame will be made available to cb
        let unmoved_frame = unsafe { &*unmoved_frame };
        frame.units = self.visible_units.iter().map(|&i| unmoved_frame.get_unit(i).expect("Unit to exist")).collect();
        cb(&frame);
    }

    fn ensure_unit_info(&mut self, id: usize) {
        if self.unit_infos[id].is_none() {
            self.unit_infos[id] = Some(UnitInfo::new(id, &self.data.get().units[id]))
        }
    }

    fn unit_invisible(&mut self, id: usize) {
        let index = self.visible_units.iter().position(|&i| i as usize == id);
        if let Some(index) = index {
            self.visible_units.swap_remove(index);
        }
    }

    pub(crate) fn handle_events(&mut self, module: &mut impl AiModule) {
        let mut commands = Commands::default();
        for i in 0..self.data.get().eventCount {
            let event: BWAPIC_Event = self.data.get().events[i as usize];
            use BWAPI_EventType_Enum::*;
            match event.type_ {
                MatchStart => {
                    let data = self.data.get();
                    self.visible_units = (0..data.initialUnitCount as usize)
                    .filter(|&i| {
                        data.units[i].exists
                            && data.units[i].type_ != types::UnitType::Unknown as i32
                    }).map(|i| i as i32)
                    .collect();
                    for &i in self.visible_units.iter() {
                        self.unit_infos[i as usize] = Some(UnitInfo::new(i as usize, &data.units[i as usize]));
                    }

                    self.with_frame(|f| module.on_start(f));
                }
                MatchFrame => {
                    self.with_frame(|f| module.on_frame(f, &mut commands));
                }
                UnitCreate => {
                    let id = event.v1 as usize;
                    self.ensure_unit_info(id);
                    self.with_frame(|frame| {
                        module.on_unit_create(
                            frame,
                            &mut commands,
                            frame.get_unit(id as i32).expect("Created Unit to exist"),
                        )
                    });
                }
                UnitDestroy => {
                    let id = event.v1;
                    self.unit_invisible(id as usize);
                    self.with_frame(|frame| {
                        module.on_unit_destroy(
                            frame,
                            &mut commands,
                            frame.get_unit(id).expect("Unit to be still available this frame"),
                        )
                    });
                    self.unit_infos[id as usize] = Option::None;
                }
                UnitDiscover => {
                    self.ensure_unit_info(event.v1 as usize);
                    self.with_frame(|frame| {
                        module.on_unit_discover(
                            frame,
                            &mut commands,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                UnitEvade => {
                    self.with_frame(|frame| {
                        module.on_unit_evade(
                            frame,
                            &mut commands,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                UnitShow => {
                    self.visible_units.push(event.v1);
                    self.ensure_unit_info(event.v1 as usize);
                    self.with_frame(|frame| {
                        module.on_unit_show(
                            frame,
                            &mut commands,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                UnitHide => {
                    self.unit_invisible(event.v1 as usize);
                    self.with_frame(|frame| {
                        module.on_unit_hide(
                            frame,
                            &mut commands,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                UnitMorph => {
                    self.with_frame(|frame| {
                        module.on_unit_morph(
                            frame,
                            &mut commands,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                UnitRenegade => {
                    self.with_frame(|frame| {
                        module.on_unit_renegade(
                            frame,
                            &mut commands,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                UnitComplete => {
                    self.ensure_unit_info(event.v1 as usize);
                    self.with_frame(|frame| {
                        module.on_unit_complete(
                            frame,
                            &mut commands,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                MatchEnd => self.with_frame(|frame| module.on_end(frame, event.v1 != 0)),
                MenuFrame => {}
                SendText => self.with_frame(|frame| {
                    module.on_send_text(frame, &mut commands, frame.event_str(event.v1 as usize))
                }),
                ReceiveText => {
                    self.with_frame(|frame| {
                        module.on_receive_text(
                            frame,
                            &mut commands,
                            frame
                                .get_player(event.v1)
                                .expect("Player could not be retrieved"),
                            frame.event_str(event.v2 as usize),
                        )
                    });
                }
                PlayerLeft => {
                    self.with_frame(|frame| {
                        module.on_player_left(
                            frame,
                            &mut commands,
                            frame
                                .get_player(event.v1)
                                .expect("Player could not be retrieved"),
                        )
                    });
                }
                NukeDetect => {
                    self.with_frame(|frame| {
                        module.on_nuke_detect(
                            frame,
                            &mut commands,
                            Position {
                                x: event.v1,
                                y: event.v2,
                            },
                        )
                    });
                }
                SaveGame => self.with_frame(|frame| {
                    module.on_save_game(frame, &mut commands, frame.event_str(event.v1 as usize))
                }),
                None => {}
            }
        }
        commands.commit(self.data.get_mut());
    }
}
