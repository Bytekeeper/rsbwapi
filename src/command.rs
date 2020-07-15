use crate::types::CoordinateType;
use crate::types::{Color, Position, TextSize};
use crate::unit::UnitCommand;
use bwapi_wrapper::*;
use std::ffi::CString;

struct CommandApplier<'a> {
    data: &'a mut BWAPI_GameData,
}

impl<'a> CommandApplier<'a> {
    fn apply_commands(&mut self, commands: &Commands) {
        for cmd in commands.commands.iter() {
            use Command::*;
            match cmd {
                DrawText {
                    ctype,
                    x,
                    y,
                    string,
                } => self.draw_text(*ctype, *x, *y, &string),
                DrawBox {
                    left,
                    right,
                    top,
                    bottom,
                    ctype,
                    solid,
                    color,
                } => self.draw_box(*ctype, *left, *right, *top, *bottom, *color, *solid),
                DrawTriangle {
                    ctype,
                    a,
                    b,
                    c,
                    color,
                    solid,
                } => self.draw_triangle(*ctype, *a, *b, *c, *color, *solid),
                DrawCircle {
                    x,
                    y,
                    radius,
                    ctype,
                    color,
                    solid,
                } => self.draw_circle(*ctype, *x, *y, *radius, *color, *solid),
                UnitCommand(cmd) => self.issue_command(*cmd),
            }
        }
    }

    fn draw_circle(
        &mut self,
        ctype: CoordinateType,
        x: i32,
        y: i32,
        radius: i32,
        color: Color,
        solid: bool,
    ) {
        self.add_shape(BWAPIC_Shape {
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
        })
    }

    fn draw_triangle(
        &mut self,
        ctype: CoordinateType,
        a: Position,
        b: Position,
        c: Position,
        color: Color,
        solid: bool,
    ) {
        self.add_shape(BWAPIC_Shape {
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

    #[allow(clippy::too_many_arguments)]
    fn draw_box(
        &mut self,
        ctype: CoordinateType,
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
        color: Color,
        solid: bool,
    ) {
        self.add_shape(BWAPIC_Shape {
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

    pub fn issue_command(&mut self, cmd: UnitCommand) {
        assert!(
            self.data.unitCommandCount < BWAPI_GameData_MAX_COMMANDS,
            "Too many unit commands"
        );
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
    DrawText {
        ctype: CoordinateType,
        x: i32,
        y: i32,
        string: String,
    },
    DrawBox {
        ctype: CoordinateType,
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
        color: Color,
        solid: bool,
    },
    DrawTriangle {
        ctype: CoordinateType,
        a: Position,
        b: Position,
        c: Position,
        color: Color,
        solid: bool,
    },
    DrawCircle {
        ctype: CoordinateType,
        x: i32,
        y: i32,
        radius: i32,
        color: Color,
        solid: bool,
    },
    UnitCommand(UnitCommand),
}

impl Commands {
    pub fn draw_text_screen<P: Into<Position>>(&mut self, position: P, string: &str) {
        self.draw_text(CoordinateType::Screen, position, string);
    }

    pub fn draw_text_map<P: Into<Position>>(&mut self, position: P, string: &str) {
        self.draw_text(CoordinateType::Map, position, string);
    }

    pub fn draw_text_mouse<P: Into<Position>>(&mut self, position: P, string: &str) {
        self.draw_text(CoordinateType::Mouse, position, string);
    }

    pub fn draw_text<P: Into<Position>>(
        &mut self,
        ctype: CoordinateType,
        position: P,
        string: &str,
    ) {
        let p = position.into();
        self.commands.push(Command::DrawText {
            ctype,
            x: p.x,
            y: p.y,
            string: string.to_owned(),
        });
    }

    pub fn draw_box_map<P: Into<Position>>(
        &mut self,
        top_left: P,
        bottom_right: P,
        color: Color,
        solid: bool,
    ) {
        self.draw_box(CoordinateType::Map, top_left, bottom_right, color, solid)
    }

    pub fn draw_box_mouse<P: Into<Position>>(
        &mut self,
        top_left: P,
        bottom_right: P,
        color: Color,
        solid: bool,
    ) {
        self.draw_box(CoordinateType::Mouse, top_left, bottom_right, color, solid)
    }

    pub fn draw_box_screen<P: Into<Position>>(
        &mut self,
        top_left: P,
        bottom_right: P,
        color: Color,
        solid: bool,
    ) {
        self.draw_box(CoordinateType::Screen, top_left, bottom_right, color, solid)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_box<P: Into<Position>>(
        &mut self,
        ctype: CoordinateType,
        top_left: P,
        bottom_right: P,
        color: Color,
        solid: bool,
    ) {
        let tl = top_left.into();
        let br = bottom_right.into();
        self.commands.push(Command::DrawBox {
            ctype,
            left: tl.x,
            right: br.x,
            top: tl.y,
            bottom: br.y,
            color,
            solid,
        })
    }

    pub fn draw_triangle_map<P: Into<Position>>(
        &mut self,
        a: P,
        b: P,
        c: P,
        color: Color,
        solid: bool,
    ) {
        self.draw_triangle(CoordinateType::Map, a, b, c, color, solid)
    }

    pub fn draw_triangle_mouse<P: Into<Position>>(
        &mut self,
        a: P,
        b: P,
        c: P,
        color: Color,
        solid: bool,
    ) {
        self.draw_triangle(CoordinateType::Mouse, a, b, c, color, solid)
    }

    pub fn draw_triangle_screen<P: Into<Position>>(
        &mut self,
        a: P,
        b: P,
        c: P,
        color: Color,
        solid: bool,
    ) {
        self.draw_triangle(CoordinateType::Screen, a, b, c, color, solid)
    }

    pub fn draw_triangle<P: Into<Position>>(
        &mut self,
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

        self.commands.push(Command::DrawTriangle {
            ctype,
            a,
            b,
            c,
            color,
            solid,
        })
    }

    pub fn draw_circle_map<P: Into<Position>>(
        &mut self,
        p: P,
        radius: i32,
        color: Color,
        solid: bool,
    ) {
        self.draw_circle(CoordinateType::Map, p, radius, color, solid)
    }

    pub fn draw_circle_screen<P: Into<Position>>(
        &mut self,
        p: P,
        radius: i32,
        color: Color,
        solid: bool,
    ) {
        self.draw_circle(CoordinateType::Screen, p, radius, color, solid)
    }

    pub fn draw_circle_mouse<P: Into<Position>>(
        &mut self,
        p: P,
        radius: i32,
        color: Color,
        solid: bool,
    ) {
        self.draw_circle(CoordinateType::Mouse, p, radius, color, solid)
    }

    pub fn draw_circle<P: Into<Position>>(
        &mut self,
        ctype: CoordinateType,
        p: P,
        radius: i32,
        color: Color,
        solid: bool,
    ) {
        let p = p.into();
        self.commands.push(Command::DrawCircle {
            ctype,
            x: p.x,
            y: p.y,
            radius,
            color,
            solid,
        });
    }

    pub fn issue_command(&mut self, cmd: UnitCommand) {
        self.commands.push(Command::UnitCommand(cmd))
    }

    pub(crate) fn commit(&self, to: &mut BWAPI_GameData) {
        CommandApplier { data: to }.apply_commands(self);
    }
}
