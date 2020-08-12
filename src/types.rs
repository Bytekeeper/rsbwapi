use bwapi_wrapper::prelude::Position;
use bwapi_wrapper::prelude::PositionTuple;
use num_derive::FromPrimitive;
use std::ffi::CStr;
use std::os::raw::c_char;

#[derive(Debug, Copy, Clone, FromPrimitive)]
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

pub(crate) fn c_str_to_str(i: &[c_char]) -> &str {
    unsafe {
        let i = &*(i as *const [c_char] as *const [u8]);
        CStr::from_bytes_with_nul_unchecked(&i[..=i.iter().position(|&c| c == 0).unwrap()])
            .to_str()
            .unwrap()
    }
}

pub struct Rectangle {
    pub tl: Position,
    pub br: Position,
}

impl Rectangle {
    pub fn new<P1: Into<Position>, P2: Into<Position>>(corner_a: P1, corner_b: P2) -> Self {
        let (mut a, mut b) = (corner_a.into(), corner_b.into());
        if a.x > b.x {
            core::mem::swap(&mut a.x, &mut b.x);
        }
        if a.y > b.y {
            core::mem::swap(&mut a.y, &mut b.y);
        }
        Rectangle { tl: a, br: b }
    }
}

impl From<(i32, i32, i32, i32)> for Rectangle {
    fn from(coords: (i32, i32, i32, i32)) -> Self {
        Rectangle::new((coords.0, coords.1), (coords.2, coords.3))
    }
}

impl From<(PositionTuple, PositionTuple)> for Rectangle {
    fn from(corners: (PositionTuple, PositionTuple)) -> Self {
        Rectangle::new(corners.0, corners.1)
    }
}
