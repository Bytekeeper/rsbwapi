use bwapi_wrapper::prelude::{Error, PositionTuple};
use num_derive::FromPrimitive;
use std::ffi::CStr;
use std::os::raw::c_char;

/// Many functions produce error code. BWAPI usually sets a error variable and returns a success flag.
/// Rsbwapi instead returns a result with the error code.
pub type BwResult<T> = Result<T, Error>;

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

pub struct Rectangle<P> {
    pub tl: P,
    pub br: P,
}

impl<P: Into<PositionTuple> + From<PositionTuple>> Rectangle<P> {
    pub fn new<P1: Into<P>, P2: Into<P>>(corner_a: P1, corner_b: P2) -> Self {
        let (mut a, mut b) = (corner_a.into().into(), corner_b.into().into());
        if a.0 > b.0 {
            core::mem::swap(&mut a.0, &mut b.0);
        }
        if a.1 > b.1 {
            core::mem::swap(&mut a.1, &mut b.1);
        }
        Rectangle {
            tl: a.into(),
            br: b.into(),
        }
    }
}

impl<P: From<PositionTuple> + Into<PositionTuple>> From<(i32, i32, i32, i32)> for Rectangle<P> {
    fn from(coords: (i32, i32, i32, i32)) -> Self {
        Rectangle::from(((coords.0, coords.1), (coords.2, coords.3)))
    }
}

impl<P: Into<PositionTuple> + From<PositionTuple>> From<(PositionTuple, PositionTuple)>
    for Rectangle<P>
{
    fn from(corners: (PositionTuple, PositionTuple)) -> Self {
        Rectangle::<P>::new(P::from(corners.0), P::from(corners.1))
    }
}

impl<P: Into<PositionTuple> + From<PositionTuple>> std::iter::IntoIterator for Rectangle<P> {
    type Item = P;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let tl: PositionTuple = self.tl.into();
        let br: PositionTuple = self.br.into();
        (tl.1..=br.1)
            .flat_map(|y| (tl.0..=br.0).map(move |x| (x, y).into()))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::TilePosition;

    #[test]
    fn should_return_all_positions_of_rectangle() {
        let rec: Rectangle<TilePosition> = Rectangle::new((0, 0), (2, 2));
        let mut iter = rec.into_iter();
        assert_eq!(Some(TilePosition { x: 0, y: 0 }), iter.next());
        assert_eq!(Some(TilePosition { x: 1, y: 0 }), iter.next());
        assert_eq!(Some(TilePosition { x: 2, y: 0 }), iter.next());
        assert_eq!(Some(TilePosition { x: 0, y: 1 }), iter.next());
        assert_eq!(Some(TilePosition { x: 1, y: 1 }), iter.next());
        assert_eq!(Some(TilePosition { x: 2, y: 1 }), iter.next());
        assert_eq!(Some(TilePosition { x: 0, y: 2 }), iter.next());
        assert_eq!(Some(TilePosition { x: 1, y: 2 }), iter.next());
        assert_eq!(Some(TilePosition { x: 2, y: 2 }), iter.next());
    }
}
