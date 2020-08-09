use crate::aimodule::AiModule;
use crate::bullet::Bullet;
use crate::command::Commands;
use crate::force::Force;
use crate::predicate::IntoPredicate;
use crate::predicate::Predicate;
use crate::region::Region;
use crate::shm::Shm;
use crate::types::c_str_to_str;
use crate::unit::UnitInfo;
use crate::*;
use bwapi_wrapper::*;
use core::cell::RefCell;
use core::cell::RefMut;
use std::collections::HashMap;

use crate::player::Player;
use crate::unit::Unit;

pub struct GameContext {
    data: Shm<BWAPI_GameData>,
    unit_infos: [Option<UnitInfo>; 10000],
    visible_units: Vec<usize>,
    static_minerals: Vec<usize>,
    static_geysers: Vec<usize>,
}

pub struct Game<'a> {
    context: &'a GameContext,
    data: &'a BWAPI_GameData,
    units: Vec<Unit<'a>>,
    infos: &'a [Option<UnitInfo>; 10000],
    pub(crate) cmd: &'a RefCell<Commands>,
    pub(crate) interceptors: RefCell<HashMap<usize, Vec<i32>>>,
    pub(crate) loaded_units: RefCell<HashMap<usize, Vec<i32>>>,
}

impl<'a> Game<'a> {
    pub fn can_build_here<P: Into<TilePosition>, B: Into<Option<&'a Unit<'a>>>>(
        &self,
        builder: B,
        position: P,
        type_: UnitType,
        check_explored: bool,
    ) -> bool {
        let builder = builder.into();
        let position = if builder.is_some() && type_.is_addon() {
            position.into() + TilePosition { x: 4, y: 1 }
        } else {
            position.into()
        };

        let lt = position;
        let rb = lt + type_.tile_size();

        if !lt.is_valid() || !(rb.to_position() - Position { x: 1, y: 1 }).is_valid() {
            return false;
        }

        if type_.is_refinery() {
            return self
                .get_geysers()
                .iter()
                .find(|x| x.get_tile_position() == position)
                .map_or(false, |x| {
                    !(x.is_visible() && x.get_type() != UnitType::Resource_Vespene_Geyser)
                });
        }

        for x in lt.x..rb.x {
            for y in lt.y..rb.y {
                if !self.is_buildable((x, y)) || (check_explored && !self.is_explored((x, y))) {
                    return false;
                }
            }
        }

        if let Some(builder) = builder {
            if !builder.get_type().is_building() {
                if !builder.has_path(lt.to_position() + type_.tile_size().to_position() / 2) {
                    return false;
                }
            } else if !builder.get_type().is_flying_building()
                && type_ != UnitType::Zerg_Nydus_Canal
                && !type_.is_flag_beacon()
            {
                return false;
            }
        }

        if type_ != UnitType::Special_Start_Location {
            let targ_pos = lt.to_position() + type_.tile_size().to_position() / 2;
            let collides_with_units = self
                .get_units_in_rectangle(
                    lt.to_position(),
                    rb.to_position(),
                    !Unit::is_flying.into_predicate()
                        & !Unit::is_loaded.into_predicate()
                        & |u: &Unit| {
                            builder.map_or(true, |b| b != u)
                                && u.get_left() <= targ_pos.x + type_.dimension_right()
                                && u.get_top() <= targ_pos.y + type_.dimension_down()
                                && u.get_right() >= targ_pos.x - type_.dimension_left()
                                && u.get_bottom() >= targ_pos.y - type_.dimension_up()
                        },
                )
                .iter()
                .any(|u| !(u.get_type().is_addon() && u.get_type().can_move()));
            if collides_with_units {
                return false;
            }

            let needs_creep = type_.requires_creep();
            if type_.get_race() != Race::Zerg || needs_creep {
                for x in lt.x..rb.x {
                    for y in lt.y..rb.y {
                        if needs_creep != self.has_creep((x, y)) {
                            return false;
                        }
                    }
                }
            }

            if type_.is_resource_depot() {
                for m in self.get_static_minerals() {
                    let tp = m.get_initial_tile_position();
                    if self.is_visible(tp) || self.is_visible((tp.x + 1, tp.y)) && !m.exists() {
                        continue;
                    }
                    if tp.x > lt.x - 5 && tp.y > lt.y - 4 && tp.x < lt.x + 7 && tp.y < lt.y + 6 {
                        return false;
                    }
                }
                for g in self.get_static_geysers() {
                    let tp = g.get_initial_tile_position();
                    if tp.x > lt.x - 7 && tp.y > lt.y - 5 && tp.x < lt.x + 7 && tp.y < lt.y + 6 {
                        return false;
                    }
                }
            }
        }

        if let Some(builder) = builder {
            if builder.get_type().is_addon()
                && type_.is_addon()
                && !self.can_build_here(
                    builder,
                    lt - TilePosition { x: 4, y: 1 },
                    builder.get_type(),
                    check_explored,
                )
            {
                return false;
            }
        }
        true
    }

    pub fn cmd(&self) -> RefMut<Commands> {
        self.cmd.borrow_mut()
    }

    pub fn countdown_timer(&self) -> i32 {
        self.data.countdownTimer
    }

    pub fn elapsed_time(&self) -> i32 {
        self.data.elapsedTime
    }

    pub fn enemies(&self) -> Vec<Player> {
        let self_ = self.self_();
        if let Some(self_) = self_ {
            self.get_players()
                .iter()
                .filter(|p| p.is_enemy(&self_))
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }

    pub fn get_average_fps(&self) -> f64 {
        self.data.averageFPS
    }

    pub fn get_player(&self, i: i32) -> Option<Player> {
        if !(0..self.data.playerCount).contains(&i) {
            None
        } else {
            let i = i as usize;
            let data = self.data.players.get(i)?;
            Some(Player::new(i, &self, &data))
        }
    }

    pub fn get_region_at<P: Into<Position>>(&self, p: P) -> Option<Region> {
        let Position { x, y } = p.into();

        let idx = self.data.mapTileRegionId[x as usize / 32][y as usize / 32];
        let region_code = if idx & 0x2000 != 0 {
            let minitile_pos_x = (x & 0x1F) / 8;
            let minitile_pos_y = (y & 0x1F) / 8;
            let index = (idx & 0x1FFF) as usize;
            if index >= self.data.mapSplitTilesMiniTileMask.len() {
                return None;
            }

            let mini_tile_mask = self.data.mapSplitTilesMiniTileMask[index];
            if mini_tile_mask as usize >= self.data.mapSplitTilesRegion1.len() {
                return None;
            }

            let minitile_shift = minitile_pos_x + minitile_pos_y * 4;
            if (mini_tile_mask >> minitile_shift) & 1 != 0 {
                self.data.mapSplitTilesRegion2[index]
            } else {
                self.data.mapSplitTilesRegion1[index]
            }
        } else {
            idx
        };
        self.get_region(region_code)
    }

    pub(crate) fn get_region(&self, id: u16) -> Option<Region> {
        if id >= self.data.regionCount as u16 {
            None
        } else {
            Some(Region::new(id, self, &self.data.regions[id as usize]))
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

    pub fn get_units_in_rectangle<
        A: Into<Position>,
        B: Into<Position>,
        P: IntoPredicate<Unit<'a>>,
    >(
        &self,
        lt: A,
        rb: B,
        pred: P,
    ) -> Vec<Unit<'a>> {
        let lt = lt.into();
        let rb = rb.into();
        let pred = pred.into_predicate();
        self.get_all_units()
            .iter()
            .filter(|u| {
                u.get_right() > lt.x
                    && u.get_left() <= rb.x
                    && u.get_bottom() > lt.y
                    && u.get_top() <= rb.y
                    && pred.test(u)
            })
            .cloned()
            .collect()
    }

    pub fn get_players(&self) -> Vec<Player> {
        (0..self.data.playerCount as usize)
            .map(|i| Player::new(i, &self, &self.data.players[i as usize]))
            .collect()
    }

    pub fn get_selected_units(&self) -> Vec<Unit> {
        (0..self.data.selectedUnitCount as usize)
            .map(|i| self.data.selectedUnits[i])
            .map(|i| self.get_unit(i).expect("Selected unit to exist"))
            .collect()
    }

    pub fn get_static_geysers(&self) -> Vec<Unit> {
        self.context
            .static_geysers
            .iter()
            .map(|&i| {
                self.get_unit(i as i32)
                    .expect("static geyser to have existed")
            })
            .collect()
    }

    pub fn get_static_minerals(&self) -> Vec<Unit> {
        self.context
            .static_minerals
            .iter()
            .map(|&i| {
                self.get_unit(i as i32)
                    .expect("static mineral to have existed")
            })
            .collect()
    }

    pub fn has_path<S: Into<Position>, D: Into<Position>>(
        &self,
        source: S,
        destination: D,
    ) -> bool {
        let source = source.into();
        let destination = destination.into();
        if source.is_valid() && destination.is_valid() {
            let rgn_a = self.get_region_at(source);
            let rgn_b = self.get_region_at(destination);
            if let (Some(rgn_a), Some(rgn_b)) = (rgn_a, rgn_b) {
                return rgn_a.get_region_group_id() == rgn_b.get_region_group_id();
            }
        }
        false
    }

    pub fn is_battle_net(&self) -> bool {
        self.data.isBattleNet
    }

    pub fn is_debug(&self) -> bool {
        self.data.isDebug
    }

    pub fn is_multiplayer(&self) -> bool {
        self.data.isMultiplayer
    }

    pub fn is_walkable<P: Into<WalkPosition>>(&self, wp: P) -> bool {
        let p = wp.into();
        self.data.isWalkable[p.x as usize][p.y as usize]
    }

    pub fn is_visible<P: Into<TilePosition>>(&self, tp: P) -> bool {
        let p = tp.into();
        self.data.isVisible[p.x as usize][p.y as usize]
    }

    pub fn is_buildable<P: Into<TilePosition>>(&self, tp: P) -> bool {
        let p = tp.into();
        self.data.isBuildable[p.x as usize][p.y as usize]
    }

    pub fn is_explored<P: Into<TilePosition>>(&self, tp: P) -> bool {
        let p = tp.into();
        self.data.isExplored[p.x as usize][p.y as usize]
    }

    pub fn is_gui_enabled(&self) -> bool {
        self.data.hasGUI
    }

    pub fn is_flag_enabled(&self, flag: Flag) -> bool {
        *self.data.flags.get(flag as usize).unwrap_or(&false)
    }

    pub fn is_lat_com_enabled(&self) -> bool {
        self.data.hasLatCom
    }

    pub fn is_paused(&self) -> bool {
        self.data.isPaused
    }

    pub fn is_replay(&self) -> bool {
        self.data.isReplay
    }

    pub fn has_creep<P: Into<TilePosition>>(&self, tp: P) -> bool {
        let p = tp.into();
        self.data.hasCreep[p.x as usize][p.y as usize]
    }

    pub fn get_ground_height<P: Into<TilePosition>>(&self, tp: P) -> i32 {
        let p = tp.into();
        self.data.getGroundHeight[p.x as usize][p.y as usize]
    }

    pub fn get_instance_number(&self) -> i32 {
        self.data.instanceID
    }

    pub fn get_key_state(&self, key: Key) -> bool {
        self.data.keyState[key as usize]
    }

    pub fn get_latency(&self) -> i32 {
        self.data.latency
    }

    pub fn get_latency_frames(&self) -> i32 {
        self.data.latencyFrames
    }

    pub fn get_latency_time(&self) -> i32 {
        self.data.latencyTime
    }

    pub fn get_mouse_position(&self) -> Position {
        Position {
            x: self.data.mouseX,
            y: self.data.mouseY,
        }
    }

    pub fn get_mouse_state(&self, button: MouseButton) -> bool {
        self.data.mouseState[button as usize]
    }

    pub fn neutral(&self) -> Player {
        self.get_player(self.data.neutral)
            .expect("Neutral player to exist")
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

    pub fn observers(&self) -> Vec<Player> {
        self.get_players()
            .iter()
            .filter(|p| p.is_observer())
            .cloned()
            .collect()
    }

    pub fn get_all_units(&self) -> &Vec<Unit<'a>> {
        &self.units
    }

    pub fn allies(&self) -> Vec<Player> {
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

    pub fn get_client_version(&self) -> i32 {
        self.data.client_version
    }

    pub fn get_events(&self) -> Vec<BWAPIC_Event> {
        (0..self.data.eventCount as usize)
            .map(|i| self.data.events[i])
            .collect()
    }

    pub fn get_force(&self, force_id: i32) -> Force {
        if !(0..self.data.forceCount).contains(&force_id) {
            panic!(format!("Invalid force id {}", force_id));
        }
        let force_players = self
            .get_players()
            .iter()
            .filter(|p| p.force_id() == force_id)
            .cloned()
            .collect();
        Force::new(
            force_id as usize,
            &self.data.forces[force_id as usize],
            force_players,
        )
    }

    pub fn get_forces(&self) -> Vec<Force> {
        (0..self.data.forceCount)
            .map(|i| self.get_force(i))
            .collect()
    }

    pub fn get_fps(&self) -> i32 {
        self.data.fps
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

    pub fn get_random_seed(&self) -> u32 {
        self.data.randomSeed
    }

    pub fn get_remaining_latency_frames(&self) -> i32 {
        self.data.remainingLatencyFrames
    }

    pub fn get_remaining_latency_time(&self) -> i32 {
        self.data.remainingLatencyTime
    }

    pub fn get_replay_frame_count(&self) -> i32 {
        self.data.replayFrameCount
    }

    pub fn get_revision(&self) -> i32 {
        self.data.revision
    }

    pub fn get_screen_position(&self) -> Position {
        Position {
            x: self.data.screenX,
            y: self.data.screenY,
        }
    }

    pub fn get_start_locations(&self) -> Vec<TilePosition> {
        (0..self.data.startLocationCount as usize)
            .map(|i| self.data.startLocations[i])
            .map(|p| TilePosition { x: p.x, y: p.y })
            .collect()
    }

    pub fn set_alliance(&mut self, other: &Player, allied: bool, allied_victory: bool) {
        if self.is_replay() || other == &self.self_().expect("Self to exist") {
            return;
        }

        self.cmd.borrow_mut().cmd.push(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::SetAllies,
            value1: other.id as i32,
            value2: if allied {
                if allied_victory {
                    2
                } else {
                    1
                }
            } else {
                0
            },
        });
    }

    pub fn set_reveal_all(&mut self, reveal: bool) -> Result<(), Error> {
        if !self.is_replay() {
            return Err(Error::Invalid_Parameter);
        }

        self.cmd.borrow_mut().cmd.push(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::SetAllies,
            value1: reveal as i32,
            value2: 0,
        });

        Ok(())
    }

    pub fn set_vision(&mut self, player: &Player, enabled: bool) -> Result<(), Error> {
        if !self.is_replay() && self.self_().ok_or(Error::Invalid_Parameter)? == *player {
            return Err(Error::Invalid_Parameter);
        }

        self.cmd.borrow_mut().cmd.push(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::SetAllies,
            value1: player.id as i32,
            value2: enabled as i32,
        });

        Ok(())
    }

    fn event_str(&self, i: usize) -> &str {
        c_str_to_str(&self.data.eventStrings[i])
    }
}

impl GameContext {
    pub(crate) fn new(data: Shm<BWAPI_GameData>) -> Self {
        GameContext {
            data,
            unit_infos: [None; 10000],
            visible_units: vec![],
            static_geysers: vec![],
            static_minerals: vec![],
        }
    }

    pub fn is_in_game(&self) -> bool {
        self.data.get().isInGame
    }

    fn with_frame(&self, cmd: &RefCell<Commands>, cb: impl FnOnce(&Game)) {
        let mut frame = Game {
            context: self,
            data: self.data.get(),
            units: vec![],
            infos: &self.unit_infos,
            cmd,
            interceptors: RefCell::new(HashMap::new()),
            loaded_units: RefCell::new(HashMap::new()),
        };
        let unmoved_frame = &frame as *const Game;
        // SAFETY: Only the infos will be modified here and only a reference of Frame will be made available to cb
        let unmoved_frame = unsafe { &*unmoved_frame };
        frame.units = self
            .visible_units
            .iter()
            .map(|&i| unmoved_frame.get_unit(i as i32).expect("Unit to exist"))
            .collect();
        cb(&frame);
    }

    fn ensure_unit_info(&mut self, id: usize) {
        if self.unit_infos[id].is_none() {
            let data = self.data.get();
            self.unit_infos[id] = Some(UnitInfo::new(id, &data.units[id]));
        }
    }

    fn unit_invisible(&mut self, id: usize) {
        let index = self.visible_units.iter().position(|&i| i as usize == id);
        if let Some(index) = index {
            self.visible_units.swap_remove(index);
        }
    }

    pub(crate) fn handle_events(&mut self, module: &mut impl AiModule) {
        let commands = RefCell::new(Commands::new());
        for i in 0..self.data.get().eventCount {
            let event: BWAPIC_Event = self.data.get().events[i as usize];
            use BWAPI_EventType_Enum::*;
            match event.type_ {
                MatchStart => {
                    let data = self.data.get();
                    self.visible_units = (0..data.initialUnitCount as usize)
                        .filter(|&i| {
                            data.units[i].exists && data.units[i].type_ != UnitType::Unknown as i32
                        })
                        .collect();

                    for &i in self.visible_units.iter() {
                        self.unit_infos[i] = Some(UnitInfo::new(i, &data.units[i]));
                        let ut = UnitType::new(data.units[i].type_);
                        if ut == UnitType::Resource_Vespene_Geyser {
                            self.static_geysers.push(i);
                        }
                        if ut.is_mineral_field() {
                            self.static_minerals.push(i);
                        }
                    }
                    self.with_frame(&commands, |f| module.on_start(f));
                    // No longer visible after the start event
                    self.visible_units.clear();
                }
                MatchFrame => {
                    self.with_frame(&commands, |f| module.on_frame(f));
                }
                UnitCreate => {
                    let id = event.v1 as usize;
                    self.ensure_unit_info(id);
                    self.with_frame(&commands, |frame| {
                        module.on_unit_create(
                            frame,
                            frame.get_unit(id as i32).expect("Created Unit to exist"),
                        )
                    });
                }
                UnitDestroy => {
                    let id = event.v1;
                    self.unit_invisible(id as usize);
                    self.with_frame(&commands, |frame| {
                        module.on_unit_destroy(
                            frame,
                            frame
                                .get_unit(id)
                                .expect("Unit to be still available this frame"),
                        )
                    });
                    self.unit_infos[id as usize] = Option::None;
                }
                UnitDiscover => {
                    self.ensure_unit_info(event.v1 as usize);
                    self.with_frame(&commands, |frame| {
                        module.on_unit_discover(
                            frame,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                UnitEvade => {
                    self.with_frame(&commands, |frame| {
                        module.on_unit_evade(
                            frame,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                UnitShow => {
                    self.visible_units.push(event.v1 as usize);
                    self.ensure_unit_info(event.v1 as usize);
                    self.with_frame(&commands, |frame| {
                        module.on_unit_show(
                            frame,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                UnitHide => {
                    self.unit_invisible(event.v1 as usize);
                    self.with_frame(&commands, |frame| {
                        module.on_unit_hide(
                            frame,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                UnitMorph => {
                    self.with_frame(&commands, |frame| {
                        module.on_unit_morph(
                            frame,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                UnitRenegade => {
                    self.with_frame(&commands, |frame| {
                        module.on_unit_renegade(
                            frame,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                UnitComplete => {
                    self.ensure_unit_info(event.v1 as usize);
                    self.with_frame(&commands, |frame| {
                        module.on_unit_complete(
                            frame,
                            frame
                                .get_unit(event.v1)
                                .expect("Unit could not be retrieved"),
                        )
                    });
                }
                MatchEnd => self.with_frame(&commands, |frame| module.on_end(frame, event.v1 != 0)),
                MenuFrame => {}
                SendText => self.with_frame(&commands, |frame| {
                    module.on_send_text(frame, frame.event_str(event.v1 as usize))
                }),
                ReceiveText => {
                    self.with_frame(&commands, |frame| {
                        module.on_receive_text(
                            frame,
                            frame
                                .get_player(event.v1)
                                .expect("Player could not be retrieved"),
                            frame.event_str(event.v2 as usize),
                        )
                    });
                }
                PlayerLeft => {
                    self.with_frame(&commands, |frame| {
                        module.on_player_left(
                            frame,
                            frame
                                .get_player(event.v1)
                                .expect("Player could not be retrieved"),
                        )
                    });
                }
                NukeDetect => {
                    self.with_frame(&commands, |frame| {
                        module.on_nuke_detect(
                            frame,
                            Position {
                                x: event.v1,
                                y: event.v2,
                            },
                        )
                    });
                }
                SaveGame => self.with_frame(&commands, |frame| {
                    module.on_save_game(frame, frame.event_str(event.v1 as usize))
                }),
                None => {}
            }
        }
        commands.into_inner().commit(self.data.get_mut());
    }
}
