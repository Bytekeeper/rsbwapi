use crate::aimodule::AiModule;
use crate::bullet::Bullet;
use crate::shm::Shm;
use crate::types::c_str_to_str;
use crate::types::TilePosition;
use crate::*;
use bwapi_wrapper::*;

use std::ffi::CString;

use crate::types::{Color, Position, TextSize};

/*
use crate::client::{
    BWAPIC_Shape, BWAPI_GameData, BWAPI_GameData_MAX_SHAPES, BWAPI_GameData_MAX_STRINGS,
    BWAPI_PlayerData, TilePosition, BWAPIC_Event,
};
use crate::client::{Client, Position};

*/
use crate::player::Player;
use crate::unit::{Unit, UnitCommand};

pub struct Game {
    data: Shm<BWAPI_GameData>,
    units: Vec<usize>,
}

#[derive(Default)]
pub struct Commands {
    draws: Vec<DrawCommand>,
    unit_commands: Vec<UnitCommand>,
}

pub enum DrawCommand {
    DrawTextScreen { x: i32, y: i32, string: String },
}

impl Commands {
    pub fn draw_text_screen<P: Into<Position>>(&mut self, position: P, string: &str) {
        let p = position.into();
        self.draws.push(DrawCommand::DrawTextScreen {
            x: p.x,
            y: p.y,
            string: string.to_string(),
        })
    }

    pub fn issue_command(&mut self, cmd: UnitCommand) {
        self.unit_commands.push(cmd)
    }
}

impl Game {
    pub(crate) fn new(data: Shm<BWAPI_GameData>) -> Self {
        Game {
            data,
            units: vec![],
        }
    }

    fn init(&mut self) {
        self.units = (0..self.data.initialUnitCount as usize)
            .filter(|&i| {
                self.data.units[i].exists
                    && self.data.units[i].type_ != types::UnitType::Unknown as i32
            })
            .collect();
    }

    fn update(&mut self) {}

    pub fn is_in_game(&self) -> bool {
        self.data.isInGame
    }

    pub(crate) fn handle_events(&mut self, module: &mut impl AiModule) {
        for i in 0..self.data.eventCount {
            let event: BWAPIC_Event = self.data.events[i as usize];
            use BWAPI_EventType_Enum::*;
            let mut commands = Commands::default();
            match event.type_ {
                MatchStart => {
                    self.init();
                    module.on_start(self);
                }
                MatchFrame => {
                    self.update();
                    module.on_frame(self, &mut commands);
                }
                UnitCreate => {
                    let id = event.v1;
                    self.units.push(id as usize);
                    module.on_unit_create(
                        self,
                        &mut commands,
                        self.get_unit(id).expect("Unit could not be retrieved"),
                    );
                }
                UnitDestroy => {
                    let id = event.v1 as usize;
                    let index = self
                        .units
                        .iter()
                        .position(|&i| i == id)
                        .expect("UnitDestroy was called with non-existant unit id");
                    self.units.swap_remove(index);
                    module.on_unit_destroy(
                        self,
                        &mut commands,
                        self.get_unit(id as i32)
                            .expect("Unit could not be retrieved"),
                    );
                }
                UnitDiscover => {
                    module.on_unit_discover(
                        self,
                        &mut commands,
                        self.get_unit(event.v1)
                            .expect("Unit could not be retrieved"),
                    );
                }
                UnitEvade => {
                    module.on_unit_evade(
                        self,
                        &mut commands,
                        self.get_unit(event.v1)
                            .expect("Unit could not be retrieved"),
                    );
                }
                UnitShow => module.on_unit_show(
                    self,
                    &mut commands,
                    self.get_unit(event.v1)
                        .expect("Unit could not be retrieved"),
                ),
                UnitHide => module.on_unit_hide(
                    self,
                    &mut commands,
                    self.get_unit(event.v1)
                        .expect("Unit could not be retrieved"),
                ),
                UnitMorph => module.on_unit_morph(
                    self,
                    &mut commands,
                    self.get_unit(event.v1)
                        .expect("Unit could not be retrieved"),
                ),
                UnitRenegade => module.on_unit_renegade(
                    self,
                    &mut commands,
                    self.get_unit(event.v1)
                        .expect("Unit could not be retrieved"),
                ),
                UnitComplete => module.on_unit_complete(
                    self,
                    &mut commands,
                    self.get_unit(event.v1)
                        .expect("Unit could not be retrieved"),
                ),
                MatchEnd => {
                    module.on_end(self, event.v1 != 0);
                }
                MenuFrame => {}
                SendText => module.on_send_text(
                    self,
                    &mut commands,
                    c_str_to_str(&self.data.eventStrings[event.v1 as usize]),
                ),
                ReceiveText => module.on_receive_text(
                    self,
                    &mut commands,
                    self.get_player(event.v1)
                        .expect("Player could not be retrieved"),
                    c_str_to_str(&self.data.eventStrings[event.v2 as usize]),
                ),
                PlayerLeft => module.on_player_left(
                    self,
                    &mut commands,
                    self.get_player(event.v1)
                        .expect("Player could not be retrieved"),
                ),
                NukeDetect => module.on_nuke_detect(
                    self,
                    &mut commands,
                    Position {
                        x: event.v1,
                        y: event.v2,
                    },
                ),
                SaveGame => module.on_save_game(
                    self,
                    &mut commands,
                    c_str_to_str(&self.data.eventStrings[event.v1 as usize]),
                ),
                None => {}
            }
            self.apply_commands(&commands)
        }
    }

    pub fn get_players(&self) -> Vec<Player> {
        (0..self.data.playerCount as usize)
            .map(|i| Player::new(i, &self.data.players[i as usize]))
            .collect()
    }

    pub fn get_player(&self, i: i32) -> Option<Player> {
        if i < 0 {
            None
        } else {
            let i = i as usize;
            let data = self.data.players.get(i)?;
            Some(Player::new(i, &data))
        }
    }

    pub fn get_all_units(&self) -> Vec<Unit> {
        self.units
            .iter()
            .map(|&i| Unit::new(i, self, &self.data.units[i]))
            .collect()
    }

    pub fn get_unit(&self, id: i32) -> Option<Unit> {
        if id < 0 {
            None
        } else {
            Some(Unit::new(id as usize, self, &self.data.units[id as usize]))
        }
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

    fn apply_commands(&mut self, commands: &Commands) {
        for draw in commands.draws.iter() {
            match draw {
                DrawCommand::DrawTextScreen { x, y, string } => {
                    self.draw_text_screen(*x, *y, string)
                }
            }
        }
        for unit_command in commands.unit_commands.iter() {
            self.issue_command(*unit_command)
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
