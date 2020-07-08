use crate::aimodule::AiModule;
use crate::shm::Shm;
use crate::*;
use bwapi_wrapper::*;


use std::ffi::{CString};





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
    pub fn new(data: Shm<BWAPI_GameData>) -> Self {
        Game {
            data,
            units: vec![]
        }
    }
    pub(crate) fn init(&mut self) {
        self.units = (0..self.data.initialUnitCount as usize)
            .filter(|&i| {
                self.data.units[i].exists
                    && self.data.units[i].type_ != types::UnitType::Unknown as i32
            })
            .collect();
    }

    pub fn is_in_game(&self) -> bool {
        self.data.isInGame
    }

    pub(crate) fn handle_events(&mut self, module: &mut impl AiModule) {
        for i in 0..self.data.eventCount {
            let event: BWAPIC_Event = self.data.events[i as usize];
            match event.type_ {
                BWAPI_EventType_Enum::MatchStart => {
                    self.init();
                    module.on_start(self);
                }
                BWAPI_EventType_Enum::MatchFrame => {
                    let mut commands = Commands::default();
                    module.on_frame(&self, &mut commands);
                    self.apply_commands(&commands)
                }
                _ => (),
            }
        }
    }

    pub fn get_players(&self) -> Vec<Player> {
        (0..self.data.playerCount as usize)
            .map(|i| Player::new(i, &self.data.players[i as usize]))
            .collect()
    }

    pub fn get_player(&self, i: usize) -> Option<Player> {
        let data = self.data.players.get(i)?;
        Some(Player::new(i, &data))
    }

    pub fn get_all_units(&self) -> Vec<Unit> {
        self.units
            .iter()
            .map(|&i| Unit::new(i, self, &self.data.units[i]))
            .collect()
    }

    pub fn enemy(&self) -> Option<Player> {
        self.get_player(self.data.enemy as usize)
    }

    pub fn self_(&self) -> Option<Player> {
        self.get_player(self.data.self_ as usize)
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
