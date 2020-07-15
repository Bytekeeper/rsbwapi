use bwapi_wrapper::*;
use num_traits::FromPrimitive;
use std::ffi::CStr;
use std::os::raw::c_char;

pub type UnitType = BWAPI_UnitTypes_Enum_Enum;
pub type CoordinateType = BWAPI_CoordinateType_Enum;

#[derive(Debug, Copy, Clone)]
pub enum Color {
    /// <summary>The default color for Player 1.</summary>
    Red = 111,

    /// <summary>The default color for Player 2.</summary>
    Blue = 165,

    /// <summary>The default color for Player 3.</summary>
    Teal = 159,

    /// <summary>The default color for Player 4.</summary>
    Purple = 164,

    /// <summary>The default color for Player 5.</summary>
    Orange = 179,

    /// <summary>The default color for Player 6.</summary>
    Brown = 19,

    /// <summary>A bright white. Note that this is lighter than Player 7's white.</summary>
    White = 255,

    /// <summary>The default color for Player 8.</summary>
    Yellow = 135,

    /// <summary>The alternate color for Player 7 on Ice tilesets.</summary>
    Green = 117,

    /// <summary>The default color for Neutral (Player 12).</summary>
    Cyan = 128,

    /// <summary>The color black</summary>
    Black = 0,

    /// <summary>The color grey</summary>
    Grey = 74,
}

#[derive(Debug)]
pub enum TextSize {
    /// <summary>The smallest text size in the game.</summary>
    Small,

    /// <summary>The standard text size, used for most things in the game such as chat messages.</summary>
    Default,

    /// <summary>A larger text size. This size is used for the in-game countdown timer seen in @CTF and @UMS game types.</summary>
    Large,

    /// <summary>The largest text size in the game.</summary>
    Huge,
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct TilePosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct WalkPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

type PositionTuple = (i32, i32);

impl Position {
    pub fn to_tile_position(&self) -> TilePosition {
        TilePosition {
            x: self.x / 32,
            y: self.y / 32,
        }
    }

    pub fn to_walk_position(&self) -> WalkPosition {
        WalkPosition {
            x: self.x / 8,
            y: self.y / 8,
        }
    }
}

impl TilePosition {
    pub fn to_position(&self) -> Position {
        Position {
            x: self.x * 32,
            y: self.y * 32,
        }
    }

    pub fn to_walk_position(&self) -> WalkPosition {
        WalkPosition {
            x: self.x * 8,
            y: self.y * 8,
        }
    }
}

impl WalkPosition {
    pub fn to_tile_position(&self) -> TilePosition {
        TilePosition {
            x: self.x / 4,
            y: self.y / 4,
        }
    }

    pub fn to_position(&self) -> Position {
        Position {
            x: self.x * 8,
            y: self.y * 8,
        }
    }
}

impl From<PositionTuple> for TilePosition {
    fn from(pos: PositionTuple) -> Self {
        Self { x: pos.0, y: pos.1 }
    }
}

impl From<PositionTuple> for Position {
    fn from(pos: (i32, i32)) -> Self {
        Self { x: pos.0, y: pos.1 }
    }
}

pub trait UnitTypeExt {
    fn is_resource_container(&self) -> bool;
    fn is_mineral_field(&self) -> bool;
}

pub(crate) fn unit_type_from(i: i32) -> UnitType {
    BWAPI_UnitTypes_Enum_Enum::from_i32(i).unwrap()
}

impl UnitTypeExt for BWAPI_UnitTypes_Enum_Enum {
    fn is_resource_container(&self) -> bool {
        self.is_mineral_field() || *self == BWAPI_UnitTypes_Enum_Enum::Resource_Vespene_Geyser
    }

    fn is_mineral_field(&self) -> bool {
        *self == BWAPI_UnitTypes_Enum_Enum::Resource_Mineral_Field
            || *self == BWAPI_UnitTypes_Enum_Enum::Resource_Mineral_Field_Type_2
            || *self == BWAPI_UnitTypes_Enum_Enum::Resource_Mineral_Field_Type_3
    }
}

pub(crate) fn c_str_to_str(i: &[c_char]) -> &str {
    unsafe {
        let i = &*(i as *const [c_char] as *const [u8]);
        CStr::from_bytes_with_nul_unchecked(&i[..=i.iter().position(|&c| c == 0).unwrap()])
            .to_str()
            .unwrap()
    }
}
