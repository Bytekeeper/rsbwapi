use crate::aimodule::AiModule;
use crate::bullet::Bullet;
use crate::shm::Shm;
use crate::types::c_str_to_str;
use crate::types::TilePosition;
use crate::*;
use bwapi_wrapper::*;
use core::ptr::NonNull;

use std::ffi::CString;

use crate::types::{Color, Position, TextSize};

use crate::player::Player;
use crate::unit::{Unit, UnitCommand};

pub struct Game {
    data: Shm<BWAPI_GameData>,
    unit_ids: Vec<usize>,
}

pub struct Frame<'a> {
    data: &'a BWAPI_GameData,
    units: Vec<Unit<'a>>,
}

struct CommandApplier<'a> {
    data: &'a mut BWAPI_GameData,
}

impl<'a> CommandApplier<'a> {
    fn apply_commands(&mut self, commands: &Commands) {
        for cmd in commands.commands.iter() {
            use Command::*;
            match cmd {
                DrawTextScreen { x, y, string } => {
                    self.draw_text_screen(*x, *y, string)
                }
                UnitCommand(cmd) => self.issue_command(*cmd)
            }
        }
    }

    fn draw_text_screen(&mut self, x: i32, y: i32, string: &str) {
        let id = self.add_string(string);
        self.add_shape(
            BWAPIC_ShapeType_Enum::Text,
            BWAPI_CoordinateType_Enum::Screen,
            x,
            y,
            0,
            0,
            id as i32,
            TextSize::Default as i32,
            Color::Black,
            false,
        );
    }

    fn add_string(&mut self, string: &str) -> usize {
        assert!(self.data.stringCount < BWAPI_GameData_MAX_STRINGS);
        let string_count = self.data.stringCount as usize;
        let string = CString::new(string).unwrap();
        let bytes = string.as_bytes_with_nul();
        let len = bytes.len();
        let dst = unsafe {
            &mut *(&mut self.data.strings[string_count][..len] as *mut [i8] as *mut [u8])
        };
        dst.copy_from_slice(bytes);
        self.data.stringCount += 1;
        string_count
    }

    fn add_shape(
        &mut self,
        shape_type: BWAPIC_ShapeType_Enum,
        coordinate_type: BWAPI_CoordinateType_Enum,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        extra1: i32,
        extra2: i32,
        color: Color,
        is_solid: bool,
    ) {
        assert!(self.data.shapeCount < BWAPI_GameData_MAX_SHAPES);
        let shape = BWAPIC_Shape {
            type_: shape_type,
            ctype: coordinate_type,
            x1,
            x2,
            y1,
            y2,
            extra1,
            extra2,
            color: color as i32,
            isSolid: is_solid,
        };
        let shape_count = self.data.shapeCount as usize;
        self.data.shapes[shape_count] = shape;
        self.data.shapeCount += 1;
    }

    pub fn issue_command(&mut self, cmd: UnitCommand) {
        let command_count = self.data.unitCommandCount as usize;
        self.data.unitCommands[command_count] = cmd;
        self.data.unitCommandCount += 1
    }
}

#[derive(Default)]
pub struct Commands {
    commands: Vec<Command>,
}

pub enum Command {
    DrawTextScreen { x: i32, y: i32, string: String },
    UnitCommand(UnitCommand)
}

impl Commands {
    pub fn draw_text_screen<P: Into<Position>>(&mut self, position: P, string: &str) {
        let p = position.into();
        self.commands.push(Command::DrawTextScreen {
            x: p.x,
            y: p.y,
            string: string.to_string(),
        })
    }

    pub fn issue_command(&mut self, cmd: UnitCommand) {
        self.commands.push(Command::UnitCommand(cmd))
    }
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
            Some(Unit::new(
                id as usize,
                NonNull::from(self),
                &self.data.units[id as usize],
            ))
        }
    }
    pub fn get_players(&self) -> Vec<Player> {
        (0..self.data.playerCount as usize)
            .map(|i| Player::new(i, &self.data.players[i as usize]))
            .collect()
    }

    pub fn map_name(&self) -> &str {
        c_str_to_str(&self.data.mapName)
    }

    pub fn get_all_units(&self) -> &Vec<Unit> {
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
            .map(|b| Bullet::new(b.id as usize, NonNull::from(self), b))
            .filter(|b| b.exists())
            .collect()
    }

    pub fn get_geysers(&self) -> Vec<Unit> {
        self.get_all_units()
            .iter()
            .filter(|u| u.get_type() == UnitType::Resource_Vespene_Geyser)
            .cloned()
            .collect()
    }

    pub fn get_ground_height(&self, tile_x: i32, tile_y: i32) -> i32 {
        self.data.getGroundHeight[tile_y as usize][tile_x as usize]
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

    pub fn has_creep<P: Into<TilePosition>>(&self, pos: P) -> bool {
        let pos = pos.into();
        self.data.hasCreep[pos.y as usize][pos.x as usize]
    }

    fn event_str(&self, i: usize) -> &str {
        c_str_to_str(&self.data.eventStrings[i])
    }
}

impl Game {
    pub(crate) fn new(data: Shm<BWAPI_GameData>) -> Self {
        Game {
            data,
            unit_ids: vec![],
        }
    }

    pub fn is_in_game(&self) -> bool {
        self.data.get().isInGame
    }

    pub(crate) fn handle_events(&mut self, module: &mut impl AiModule) {
        let data = self.data.get();
        let mut frame = Frame {
            data,
            units: vec![],
        };
        let frame_ptr = NonNull::from(&frame);
        let units = self
            .unit_ids
            .iter()
            .map(|&i| Unit::new(i, frame_ptr, &data.units[i]))
            .collect();
        frame.units = units;
        let mut commands = Commands::default();
        for i in 0..data.eventCount {
            let event: BWAPIC_Event = data.events[i as usize];
            use BWAPI_EventType_Enum::*;
            match event.type_ {
                MatchStart => {
                    self.unit_ids = (0..data.initialUnitCount as usize)
                        .filter(|&i| {
                            data.units[i].exists
                                && data.units[i].type_ != types::UnitType::Unknown as i32
                        })
                        .collect();
                    module.on_start(&frame);
                }
                MatchFrame => {
                    module.on_frame(&frame, &mut commands);
                }
                UnitCreate => {
                    let id = event.v1 as usize;
                    self.unit_ids.push(id);
                    let unit = Unit::new(id, frame_ptr, &data.units[id]);
                    frame.units.push(unit);
                    module.on_unit_create(&frame, &mut commands, unit);
                }
                UnitDestroy => {
                    let id = event.v1 as usize;
                    let index = self
                        .unit_ids
                        .iter()
                        .position(|&i| i == id)
                        .expect("UnitDestroy was called with non-existant unit id");
                    self.unit_ids.swap_remove(index);
                    let unit = frame.units.swap_remove(index);
                    module.on_unit_destroy(&frame, &mut commands, unit);
                }
                UnitDiscover => {
                    module.on_unit_discover(
                        &frame,
                        &mut commands,
                        frame
                            .get_unit(event.v1)
                            .expect("Unit could not be retrieved"),
                    );
                }
                UnitEvade => {
                    module.on_unit_evade(
                        &frame,
                        &mut commands,
                        frame
                            .get_unit(event.v1)
                            .expect("Unit could not be retrieved"),
                    );
                }
                UnitShow => module.on_unit_show(
                    &frame,
                    &mut commands,
                    frame
                        .get_unit(event.v1)
                        .expect("Unit could not be retrieved"),
                ),
                UnitHide => module.on_unit_hide(
                    &frame,
                    &mut commands,
                    frame
                        .get_unit(event.v1)
                        .expect("Unit could not be retrieved"),
                ),
                UnitMorph => module.on_unit_morph(
                    &frame,
                    &mut commands,
                    frame
                        .get_unit(event.v1)
                        .expect("Unit could not be retrieved"),
                ),
                UnitRenegade => module.on_unit_renegade(
                    &frame,
                    &mut commands,
                    frame
                        .get_unit(event.v1)
                        .expect("Unit could not be retrieved"),
                ),
                UnitComplete => module.on_unit_complete(
                    &frame,
                    &mut commands,
                    frame
                        .get_unit(event.v1)
                        .expect("Unit could not be retrieved"),
                ),
                MatchEnd => {
                    module.on_end(&frame, event.v1 != 0);
                }
                MenuFrame => {}
                SendText => {
                    module.on_send_text(&frame, &mut commands, frame.event_str(event.v1 as usize))
                }
                ReceiveText => module.on_receive_text(
                    &frame,
                    &mut commands,
                    frame
                        .get_player(event.v1)
                        .expect("Player could not be retrieved"),
                    frame.event_str(event.v2 as usize),
                ),
                PlayerLeft => module.on_player_left(
                    &frame,
                    &mut commands,
                    frame
                        .get_player(event.v1)
                        .expect("Player could not be retrieved"),
                ),
                NukeDetect => module.on_nuke_detect(
                    &frame,
                    &mut commands,
                    Position {
                        x: event.v1,
                        y: event.v2,
                    },
                ),
                SaveGame => {
                    module.on_save_game(&frame, &mut commands, frame.event_str(event.v1 as usize))
                }
                None => {}
            }
        }
        let mut command_applier = CommandApplier {
            data: self.data.get_mut(),
        };
        command_applier.apply_commands(&commands);
    }
}
