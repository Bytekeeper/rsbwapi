use crate::types::{Color, TextSize};
use crate::{Game, Player};
use bwapi_wrapper::prelude::*;
use bwapi_wrapper::*;
use core::cell::RefMut;
use std::ffi::CString;
use std::path::Path;

struct CommandApplier<'a> {
    data: &'a mut BWAPI_GameData,
}

impl<'a> CommandApplier<'a> {
    fn add_commands<T: Copy>(
        limit: i32,
        count: &mut i32,
        src: &[T],
        dst: &mut [T],
    ) -> Result<(), ()> {
        let copy_amount = ((limit - *count) as usize).min(src.len());
        if *count + copy_amount as i32 >= limit {
            return Err(());
        }
        let command_count = *count as usize;
        dst[command_count..command_count + copy_amount].copy_from_slice(src);
        *count += copy_amount as i32;
        Ok(())
    }

    fn apply_commands(&mut self, commands: Commands) {
        CommandApplier::add_commands(
            BWAPI_GameData_MAX_COMMANDS,
            &mut self.data.commandCount,
            &commands.game_commands,
            &mut self.data.commands,
        )
        .map_err(|_| "Too many game commmands")
        .unwrap();

        CommandApplier::add_commands(
            BWAPI_GameData_MAX_UNIT_COMMANDS,
            &mut self.data.unitCommandCount,
            &commands.unit_commands,
            &mut self.data.unitCommands,
        )
        .map_err(|_| "Too many unit commmands")
        .unwrap();

        for cmd in commands.commands.iter() {
            use Command::*;
            match cmd {
                DrawText {
                    ctype,
                    x,
                    y,
                    string,
                } => self.draw_text(*ctype, *x, *y, string),
                SendText { message, to_allies } => self.send_text(message, *to_allies),
                SetMap(map_file_name) => self.set_map(map_file_name),
            }
        }

        for shape in commands.shapes.iter() {
            self.add_shape(*shape);
        }
    }

    pub fn set_map(&mut self, map_file_name: &str) {
        let string_index = self.add_string(map_file_name);
        self.add_command(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::SetMap,
            value1: string_index as i32,
            value2: 0,
        })
    }

    fn send_text(&mut self, message: &str, to_allies: bool) {
        let string_index = self.add_string(message);
        self.add_command(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::SendText,
            value1: string_index as i32,
            value2: to_allies as i32,
        })
    }

    fn draw_text(&mut self, ctype: CoordinateType, x1: i32, y1: i32, string: &str) {
        let id = self.add_string(string);
        let shape = BWAPIC_Shape {
            type_: BWAPIC_ShapeType_Enum::Text,
            ctype,
            x1,
            x2: 0,
            y1,
            y2: 0,
            extra1: id as i32,
            extra2: TextSize::Default as i32,
            color: Color::Black as i32,
            isSolid: false,
        };

        self.add_shape(shape);
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

    fn add_shape(&mut self, shape: BWAPIC_Shape) {
        assert!(
            self.data.shapeCount < BWAPI_GameData_MAX_SHAPES,
            "Too many shapes"
        );
        let shape_count = self.data.shapeCount as usize;
        self.data.shapes[shape_count] = shape;
        self.data.shapeCount += 1;
    }

    fn add_command(&mut self, cmd: BWAPIC_Command) {
        assert!(
            self.data.commandCount < BWAPI_GameData_MAX_COMMANDS,
            "Too many commands"
        );
        let command_count = self.data.commandCount as usize;
        self.data.commands[command_count] = cmd;
        self.data.commandCount += 1;
    }
}

#[derive(Default)]
pub(crate) struct Commands {
    commands: Vec<Command>,
    game_commands: Vec<BWAPIC_Command>,
    unit_commands: Vec<UnitCommand>,
    shapes: Vec<BWAPIC_Shape>,
}

pub(crate) enum Command {
    DrawText {
        ctype: CoordinateType,
        x: i32,
        y: i32,
        string: String,
    },
    SendText {
        message: String,
        to_allies: bool,
    },
    SetMap(String),
}

impl Commands {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn commit(self, to: &mut BWAPI_GameData) {
        CommandApplier { data: to }.apply_commands(self);
    }
}

impl Game {
    fn cmd(&'_ self) -> RefMut<'_, Commands> {
        self.inner.cmd.borrow_mut()
    }

    pub fn send_text_ex(&self, to_allies: bool, message: &str) {
        self.cmd().commands.push(Command::SendText {
            to_allies,
            message: message.to_owned(),
        });
    }

    pub fn send_text(&self, message: &str) {
        self.send_text_ex(false, message);
    }

    pub fn set_map(&self, map_file_name: &str) -> Result<(), Error> {
        if map_file_name.len() >= 260 || map_file_name.is_empty() {
            return Err(Error::Invalid_Parameter);
        }

        if !Path::new(map_file_name).exists() {
            return Err(Error::File_Not_Found);
        }

        self.cmd()
            .commands
            .push(Command::SetMap(map_file_name.to_owned()));
        Ok(())
    }

    pub fn draw_text_screen<P: Into<Position>>(&self, position: P, string: &str) {
        self.draw_text(CoordinateType::Screen, position, string);
    }

    pub fn draw_text_map<P: Into<Position>>(&self, position: P, string: &str) {
        self.draw_text(CoordinateType::Map, position, string);
    }

    pub fn draw_text_mouse<P: Into<Position>>(&self, position: P, string: &str) {
        self.draw_text(CoordinateType::Mouse, position, string);
    }

    pub fn draw_text<P: Into<Position>>(&self, ctype: CoordinateType, position: P, string: &str) {
        let p = position.into();
        self.cmd().commands.push(Command::DrawText {
            ctype,
            x: p.x,
            y: p.y,
            string: string.to_owned(),
        });
    }

    pub fn draw_line_screen<P: Into<Position>>(&self, a: P, b: P, color: Color) {
        self.draw_line(CoordinateType::Screen, a, b, color)
    }

    pub fn draw_line_map<P: Into<Position>>(&self, a: P, b: P, color: Color) {
        self.draw_line(CoordinateType::Map, a, b, color)
    }

    pub fn draw_line_mouse<P: Into<Position>>(&self, a: P, b: P, color: Color) {
        self.draw_line(CoordinateType::Mouse, a, b, color)
    }

    pub fn draw_dot_screen<P: Into<Position>>(&self, p: P, color: Color) {
        self.draw_dot(CoordinateType::Screen, p, color)
    }

    pub fn draw_dot_map<P: Into<Position>>(&self, p: P, color: Color) {
        self.draw_dot(CoordinateType::Map, p, color)
    }

    pub fn draw_dot_mouse<P: Into<Position>>(&self, p: P, color: Color) {
        self.draw_dot(CoordinateType::Mouse, p, color)
    }

    pub fn draw_dot<P: Into<Position>>(&self, ctype: CoordinateType, p: P, color: Color) {
        let Position { x, y } = p.into();
        self.cmd().shapes.push(BWAPIC_Shape {
            type_: BWAPIC_ShapeType_Enum::Dot,
            ctype,
            x1: x,
            y1: y,
            x2: 0,
            y2: 0,
            extra1: 0,
            extra2: 0,
            color: color as i32,
            isSolid: false,
        })
    }

    pub fn draw_triangle_map<P: Into<Position>>(
        &self,
        a: P,
        b: P,
        c: P,
        color: Color,
        solid: bool,
    ) {
        self.draw_triangle(CoordinateType::Map, a, b, c, color, solid)
    }

    pub fn draw_triangle_mouse<P: Into<Position>>(
        &self,
        a: P,
        b: P,
        c: P,
        color: Color,
        solid: bool,
    ) {
        self.draw_triangle(CoordinateType::Mouse, a, b, c, color, solid)
    }

    pub fn draw_triangle_screen<P: Into<Position>>(
        &self,
        a: P,
        b: P,
        c: P,
        color: Color,
        solid: bool,
    ) {
        self.draw_triangle(CoordinateType::Screen, a, b, c, color, solid)
    }

    pub fn draw_triangle<P: Into<Position>>(
        &self,
        ctype: CoordinateType,
        a: P,
        b: P,
        c: P,
        color: Color,
        solid: bool,
    ) {
        let a = a.into();
        let b = b.into();
        let c = c.into();

        self.cmd().shapes.push(BWAPIC_Shape {
            type_: BWAPIC_ShapeType_Enum::Triangle,
            ctype,
            x1: a.x,
            y1: a.y,
            x2: b.x,
            y2: b.y,
            extra1: c.x,
            extra2: c.y,
            color: color as i32,
            isSolid: solid,
        })
    }

    pub fn draw_circle_map<P: Into<Position>>(&self, p: P, radius: i32, color: Color, solid: bool) {
        self.draw_circle(CoordinateType::Map, p, radius, color, solid)
    }

    pub fn draw_circle_screen<P: Into<Position>>(
        &self,
        p: P,
        radius: i32,
        color: Color,
        solid: bool,
    ) {
        self.draw_circle(CoordinateType::Screen, p, radius, color, solid)
    }

    pub fn draw_circle_mouse<P: Into<Position>>(
        &self,
        p: P,
        radius: i32,
        color: Color,
        solid: bool,
    ) {
        self.draw_circle(CoordinateType::Mouse, p, radius, color, solid)
    }

    pub fn draw_circle<P: Into<Position>>(
        &self,
        ctype: CoordinateType,
        p: P,
        radius: i32,
        color: Color,
        solid: bool,
    ) {
        let Position { x, y } = p.into();
        self.cmd().shapes.push(BWAPIC_Shape {
            type_: BWAPIC_ShapeType_Enum::Circle,
            ctype,
            x1: x,
            y1: y,
            x2: 0,
            y2: 0,
            extra1: radius,
            extra2: 0,
            color: color as i32,
            isSolid: solid,
        });
    }

    pub fn draw_ellipse_screen<P: Into<Position>>(
        &self,
        p: P,
        xrad: i32,
        yrad: i32,
        color: Color,
        solid: bool,
    ) {
        self.draw_ellipse(CoordinateType::Screen, p, xrad, yrad, color, solid)
    }

    pub fn draw_ellipse_map<P: Into<Position>>(
        &self,
        p: P,
        xrad: i32,
        yrad: i32,
        color: Color,
        solid: bool,
    ) {
        self.draw_ellipse(CoordinateType::Map, p, xrad, yrad, color, solid)
    }

    pub fn draw_ellipse_mouse<P: Into<Position>>(
        &self,
        p: P,
        xrad: i32,
        yrad: i32,
        color: Color,
        solid: bool,
    ) {
        self.draw_ellipse(CoordinateType::Mouse, p, xrad, yrad, color, solid)
    }

    pub fn draw_ellipse<P: Into<Position>>(
        &self,
        ctype: CoordinateType,
        p: P,
        xrad: i32,
        yrad: i32,
        color: Color,
        solid: bool,
    ) {
        let Position { x, y } = p.into();
        self.cmd().shapes.push(BWAPIC_Shape {
            type_: BWAPIC_ShapeType_Enum::Ellipse,
            ctype,
            x1: x,
            y1: y,
            x2: 0,
            y2: 0,
            extra1: xrad,
            extra2: yrad,
            color: color as i32,
            isSolid: solid,
        });
    }

    pub fn draw_line<P: Into<Position>>(&self, ctype: CoordinateType, a: P, b: P, color: Color) {
        let a = a.into();
        let b = b.into();
        self.cmd().shapes.push(BWAPIC_Shape {
            type_: BWAPIC_ShapeType_Enum::Line,
            ctype,
            x1: a.x,
            y1: a.y,
            x2: b.x,
            y2: b.y,
            extra1: 0,
            extra2: 0,
            color: color as i32,
            isSolid: false,
        });
    }

    pub fn draw_box_map<P: Into<Position>>(
        &self,
        top_left: P,
        bottom_right: P,
        color: Color,
        solid: bool,
    ) {
        self.draw_box(CoordinateType::Map, top_left, bottom_right, color, solid)
    }

    pub fn draw_box_mouse<P: Into<Position>>(
        &self,
        top_left: P,
        bottom_right: P,
        color: Color,
        solid: bool,
    ) {
        self.draw_box(CoordinateType::Mouse, top_left, bottom_right, color, solid)
    }

    pub fn draw_box_screen<P: Into<Position>>(
        &self,
        top_left: P,
        bottom_right: P,
        color: Color,
        solid: bool,
    ) {
        self.draw_box(CoordinateType::Screen, top_left, bottom_right, color, solid)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_box<P: Into<Position>>(
        &self,
        ctype: CoordinateType,
        top_left: P,
        bottom_right: P,
        color: Color,
        solid: bool,
    ) {
        let Position { x: left, y: top } = top_left.into();
        let Position {
            x: right,
            y: bottom,
        } = bottom_right.into();
        self.cmd().shapes.push(BWAPIC_Shape {
            type_: BWAPIC_ShapeType_Enum::Box,
            ctype,
            x1: left,
            x2: right,
            y1: top,
            y2: bottom,
            extra1: 0,
            extra2: 0,
            color: color as i32,
            isSolid: solid,
        })
    }

    pub fn issue_command(&self, cmd: UnitCommand) {
        self.cmd().unit_commands.push(cmd)
    }

    pub fn set_alliance(&mut self, other: &Player, allied: bool, allied_victory: bool) {
        if self.is_replay() || other == &self.self_().expect("Self to exist") {
            return;
        }

        self.cmd().game_commands.push(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::SetAllies,
            value1: other.id as i32,
            value2: if allied {
                if allied_victory { 2 } else { 1 }
            } else {
                0
            },
        });
    }

    pub fn set_reveal_all(&mut self, reveal: bool) -> Result<(), Error> {
        if !self.is_replay() {
            return Err(Error::Invalid_Parameter);
        }

        self.cmd().game_commands.push(BWAPIC_Command {
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

        self.cmd().game_commands.push(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::SetAllies,
            value1: player.id as i32,
            value2: enabled as i32,
        });

        Ok(())
    }

    pub fn leave_game(&self) {
        self.cmd().game_commands.push(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::LeaveGame,
            value1: 0,
            value2: 0,
        });
    }

    pub fn pause_game(&self) {
        self.cmd().game_commands.push(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::PauseGame,
            value1: 0,
            value2: 0,
        });
    }

    pub fn ping_minimap<P: Into<Position>>(&mut self, p: P) {
        let p = p.into();
        self.cmd().game_commands.push(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::PingMinimap,
            value1: p.x,
            value2: p.y,
        });
    }

    pub fn restart_game(&mut self) {
        self.cmd().game_commands.push(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::RestartGame,
            value1: 0,
            value2: 0,
        });
    }

    pub fn enable_flag(&mut self, flag: i32) {
        self.cmd().game_commands.push(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::EnableFlag,
            value1: flag,
            value2: 0,
        });
    }

    pub fn set_frame_skip(&mut self, frame_skip: i32) {
        if frame_skip > 0 {
            self.cmd().game_commands.push(BWAPIC_Command {
                type_: BWAPIC_CommandType_Enum::SetFrameSkip,
                value1: frame_skip,
                value2: 0,
            });
        }
    }

    pub fn set_gui(&mut self, enabled: bool) {
        self.cmd().game_commands.push(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::SetGui,
            value1: enabled as i32,
            value2: 0,
        });
    }

    pub fn set_lat_com(&mut self, enabled: bool) {
        self.cmd().game_commands.push(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::SetLatCom,
            value1: enabled as i32,
            value2: 0,
        });
    }

    pub fn set_local_speed(&mut self, speed: i32) {
        self.cmd().game_commands.push(BWAPIC_Command {
            type_: BWAPIC_CommandType_Enum::SetLocalSpeed,
            value1: speed,
            value2: 0,
        });
    }
}
