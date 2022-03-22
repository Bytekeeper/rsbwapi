use bwapi_wrapper::prelude::{Error, ScaledPosition, TilePosition, WalkPosition};
use derive_more::{Add, AddAssign, Sub, SubAssign};
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

pub(crate) fn c_str_to_str(i: &[c_char]) -> String {
    unsafe {
        let i = &*(i as *const [c_char] as *const [u8]);
        CStr::from_bytes_with_nul_unchecked(&i[..=i.iter().position(|&c| c == 0).unwrap()])
            .to_string_lossy()
            .to_string()
    }
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Add, Sub, AddAssign, SubAssign)]
pub struct Rectangle<P> {
    pub tl: P,
    pub br: P,
}

impl<const N: i32> Rectangle<ScaledPosition<N>> {
    pub fn new<P1: Into<ScaledPosition<N>>, P2: Into<ScaledPosition<N>>>(
        corner_a: P1,
        corner_b: P2,
    ) -> Self {
        let (mut a, mut b) = (corner_a.into(), corner_b.into());
        if a.x > b.x {
            core::mem::swap(&mut a.x, &mut b.x);
        }
        if a.y > b.y {
            core::mem::swap(&mut a.y, &mut b.y);
        }
        Rectangle { tl: a, br: b }
    }

    pub fn border(&self) -> Vec<ScaledPosition<N>> {
        (self.tl.x..=self.br.x)
            .map(|x| ScaledPosition::<N>::new(x, self.tl.y))
            .chain((self.tl.y + 1..self.br.y).map(|y| ScaledPosition::<N>::new(self.br.x, y)))
            .chain(
                (self.tl.x..=self.br.x)
                    .rev()
                    .map(|x| ScaledPosition::<N>::new(x, self.br.y)),
            )
            .chain(
                (self.tl.y + 1..self.br.y)
                    .rev()
                    .map(|y| ScaledPosition::<N>::new(self.tl.x, y)),
            )
            .collect()
    }

    pub fn extrude(self, amount: i32) -> Self {
        Self::new(self.tl - (amount, amount), self.br + (amount, amount))
    }

    pub fn shrink(self, amount: i32) -> Self {
        self.extrude(-amount)
    }

    pub fn envelops<const M: i32>(&self, other: Rectangle<ScaledPosition<M>>) -> bool {
        self.tl.x * N <= other.tl.x * M
            && self.tl.y * N <= other.tl.y * M
            && self.br.x * N >= other.br.x * M
            && self.br.y * N >= other.br.y * M
    }

    pub fn contains<const M: i32>(&self, pos: ScaledPosition<M>) -> bool {
        self.tl.x * N <= pos.x * M
            && self.tl.y * N <= pos.y * M
            && self.br.x * N >= pos.x * M
            && self.br.y * N >= pos.y * M
    }

    pub fn resize_to_contain(mut self, pos: ScaledPosition<N>) -> Self {
        self.tl.x = self.tl.x.min(pos.x);
        self.br.x = self.br.x.min(pos.x);
        self.tl.y = self.tl.y.min(pos.y);
        self.br.y = self.br.y.min(pos.y);
        self
    }

    pub fn width(&self) -> i32 {
        self.br.x - self.tl.x + 1
    }

    pub fn height(&self) -> i32 {
        self.br.y - self.tl.y + 1
    }
}

impl Rectangle<TilePosition> {
    pub fn to_walk_rect(self) -> Rectangle<WalkPosition> {
        Rectangle {
            tl: self.tl.to_walk_position(),
            br: self.br.to_walk_position(),
        }
    }
}

impl<const N: i32> From<(i32, i32, i32, i32)> for Rectangle<ScaledPosition<N>> {
    fn from(coords: (i32, i32, i32, i32)) -> Self {
        Self::new((coords.0, coords.1), (coords.2, coords.3))
    }
}

impl<const N: i32> std::iter::IntoIterator for Rectangle<ScaledPosition<N>> {
    type Item = ScaledPosition<N>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        (self.tl.y..=self.br.y)
            .flat_map(|y| (self.tl.x..=self.br.x).map(move |x| ScaledPosition::<N>::new(x, y)))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;

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

    #[test]
    fn should_return_border() {
        let rec: Rectangle<WalkPosition> = Rectangle::new((3, 4), (5, 6));
        assert_eq!(
            rec.border(),
            [
                ScaledPosition { x: 3, y: 4 },
                ScaledPosition { x: 4, y: 4 },
                ScaledPosition { x: 5, y: 4 },
                ScaledPosition { x: 5, y: 5 },
                ScaledPosition { x: 5, y: 6 },
                ScaledPosition { x: 4, y: 6 },
                ScaledPosition { x: 3, y: 6 },
                ScaledPosition { x: 3, y: 5 },
            ]
        );
    }
}
