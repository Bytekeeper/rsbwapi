use core::ops::{Add, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub};
use derive_more::{Add, AddAssign, Display, From, Sub, SubAssign};

#[derive(
    Default, Debug, Display, Copy, Clone, Eq, PartialEq, Add, Sub, AddAssign, SubAssign, From,
)]
#[display(fmt = "Position<{}> ({}, {})", N, x, y)]
pub struct ScaledPosition<const N: i32> {
    pub x: i32,
    pub y: i32,
}

pub type Position = ScaledPosition<1>;
pub type WalkPosition = ScaledPosition<8>;
pub type TilePosition = ScaledPosition<32>;

#[derive(Debug, Copy, Clone, PartialEq, Add, Sub, AddAssign, SubAssign, From)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

pub type PositionTuple = (i32, i32);
pub const ORIGIN: Position = Position { x: 0, y: 0 };
pub const WALK_POSITION_8_DIR: [WalkPosition; 8] = dir_8();
pub const WALK_POSITION_4_DIR: [WalkPosition; 4] = dir_4();

pub const fn dir_4<const N: i32>() -> [ScaledPosition<N>; 4] {
    [
        ScaledPosition::<N>::new(0, -1),
        ScaledPosition::<N>::new(-1, 0),
        ScaledPosition::<N>::new(1, 0),
        ScaledPosition::<N>::new(0, 1),
    ]
}

pub const fn dir_8<const N: i32>() -> [ScaledPosition<N>; 8] {
    [
        ScaledPosition::<N>::new(-1, -1),
        ScaledPosition::<N>::new(0, -1),
        ScaledPosition::<N>::new(1, -1),
        ScaledPosition::<N>::new(1, 0),
        ScaledPosition::<N>::new(1, 1),
        ScaledPosition::<N>::new(0, 1),
        ScaledPosition::<N>::new(-1, 1),
        ScaledPosition::<N>::new(-1, 0),
    ]
}

const fn pos_to_pos<const I: i32, const O: i32>(pos: ScaledPosition<I>) -> ScaledPosition<O> {
    ScaledPosition {
        x: pos.x * I / O,
        y: pos.y * I / O,
    }
}

impl Position {
    pub const fn to_tile_position(self) -> TilePosition {
        pos_to_pos(self)
    }

    pub const fn to_walk_position(self) -> WalkPosition {
        pos_to_pos(self)
    }

    pub fn get_approx_distance<P: Into<Position>>(&self, other: P) -> i32 {
        let p = other.into();
        let mut max = (self.x - p.x).abs();
        let mut min = (self.y - p.y).abs();

        if max < min {
            core::mem::swap(&mut max, &mut min);
        }

        if min <= (max >> 2) {
            return max;
        }

        let min_calc = (3 * min) >> 3;
        (min_calc >> 5) + min_calc + max - (max >> 4) - (max >> 6)
    }
}

impl TilePosition {
    pub const fn to_position(self) -> Position {
        pos_to_pos(self)
    }

    pub const fn to_walk_position(self) -> WalkPosition {
        pos_to_pos(self)
    }
}

impl WalkPosition {
    pub const fn to_tile_position(self) -> TilePosition {
        pos_to_pos(self)
    }

    pub const fn to_position(self) -> Position {
        pos_to_pos(self)
    }
}

impl Vector2D {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

pub trait PositionValidator {
    fn is_valid<const N: i32>(&self, pos: ScaledPosition<N>) -> bool;
}

impl<const N: i32> ScaledPosition<N> {
    pub fn new_checked(validator: &impl PositionValidator, x: i32, y: i32) -> Option<Self> {
        let pos = Self::new(x, y);
        if validator.is_valid(pos) {
            Some(pos)
        } else {
            None
        }
    }

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn is_valid(self, validator: &impl PositionValidator) -> bool {
        validator.is_valid(self)
    }

    pub const fn distance_squared(&self, other: Self) -> u32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy) as u32
    }

    pub fn distance(&self, other: Self) -> f64 {
        (self.distance_squared(other) as f64).sqrt()
    }

    pub fn chebyshev_distance(&self, other: Self) -> u32 {
        (self.x - other.x).abs().max((self.y - other.y).abs()) as u32
    }
}

impl<const N: i32> From<ScaledPosition<N>> for PositionTuple {
    fn from(pos: ScaledPosition<N>) -> Self {
        (pos.x, pos.y)
    }
}

impl<const N: i32> Mul<i32> for ScaledPosition<N> {
    type Output = Self;

    fn mul(self, other: i32) -> Self::Output {
        Self::Output {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl<const N: i32> Mul<ScaledPosition<N>> for i32 {
    type Output = ScaledPosition<N>;

    fn mul(self, other: ScaledPosition<N>) -> Self::Output {
        Self::Output {
            x: self * other.x,
            y: self * other.y,
        }
    }
}

impl<const N: i32> MulAssign<i32> for ScaledPosition<N> {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<const N: i32> DivAssign<i32> for ScaledPosition<N> {
    fn div_assign(&mut self, rhs: i32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl<const N: i32> Div<i32> for ScaledPosition<N> {
    type Output = Self;
    fn div(self, other: i32) -> Self::Output {
        Self::Output {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl<const N: i32> Sub<PositionTuple> for ScaledPosition<N> {
    type Output = Self;

    fn sub(self, other: PositionTuple) -> Self::Output {
        Self::Output {
            x: self.x - other.0,
            y: self.y - other.1,
        }
    }
}

impl<const N: i32> Add<PositionTuple> for ScaledPosition<N> {
    type Output = Self;

    fn add(self, other: PositionTuple) -> Self::Output {
        Self::Output {
            x: self.x + other.0,
            y: self.y + other.1,
        }
    }
}

pub trait PositionIndexed<const N: i32> {
    // empty
}

impl<T: PositionIndexed<N>, const N: i32, const M: usize> Index<ScaledPosition<N>> for Vec<[T; M]> {
    type Output = T;

    fn index(&self, index: ScaledPosition<N>) -> &Self::Output {
        &(self as &Vec<[T; M]>)[index.y as usize][index.x as usize]
    }
}

impl<T: PositionIndexed<N>, const N: i32, const M: usize> IndexMut<ScaledPosition<N>>
    for Vec<[T; M]>
{
    fn index_mut(&mut self, index: ScaledPosition<N>) -> &mut Self::Output {
        &mut (self as &mut Vec<[T; M]>)[index.y as usize][index.x as usize]
    }
}

impl<T: PositionIndexed<N>, const N: i32, const M: usize> Index<ScaledPosition<N>> for [[T; M]] {
    type Output = T;

    fn index(&self, index: ScaledPosition<N>) -> &Self::Output {
        &(self as &[[T; M]])[index.y as usize][index.x as usize]
    }
}

impl<T: PositionIndexed<N>, const N: i32, const M: usize> IndexMut<ScaledPosition<N>>
    for [[T; M]]
{
    fn index_mut(&mut self, index: ScaledPosition<N>) -> &mut Self::Output {
        &mut (self as &mut [[T; M]])[index.y as usize][index.x as usize]
    }
}
