use core::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ScaledPosition<const N: i32> {
    pub x: i32,
    pub y: i32,
}

pub type Position = ScaledPosition<1>;
pub type WalkPosition = ScaledPosition<8>;
pub type TilePosition = ScaledPosition<32>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

pub type PositionTuple = (i32, i32);
pub const ORIGIN: Position = Position { x: 0, y: 0 };

fn pos_to_pos<const I: i32, const O: i32>(pos: ScaledPosition<I>) -> ScaledPosition<O> {
    ScaledPosition {
        x: pos.x * I / O,
        y: pos.y * I / O,
    }
}

impl Position {
    pub fn to_tile_position(self) -> TilePosition {
        pos_to_pos(self)
    }

    pub fn to_walk_position(self) -> WalkPosition {
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
    pub fn to_position(self) -> Position {
        pos_to_pos(self)
    }

    pub fn to_walk_position(self) -> WalkPosition {
        pos_to_pos(self)
    }
}

impl WalkPosition {
    pub fn to_tile_position(self) -> TilePosition {
        pos_to_pos(self)
    }

    pub fn to_position(self) -> Position {
        pos_to_pos(self)
    }
}

impl Vector2D {
    pub fn new(x: f64, y: f64) -> Option<Self> {
        if x == 0.0 && y == 0.0 {
            None
        } else {
            Some(Self { x, y })
        }
    }
}

pub trait PositionValidator {
    fn is_valid<const N: i32>(&self, pos: &ScaledPosition<N>) -> bool;
}

impl<const N: i32> ScaledPosition<N> {
    pub fn new(x: i32, y: i32) -> Option<Self> {
        Some(Self { x, y })
    }

    pub fn is_valid(&self, validator: &impl PositionValidator) -> bool {
        validator.is_valid(self)
    }

    pub fn distance_squared(&self, other: Self) -> u32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy) as u32
    }

    pub fn distance(&self, other: Self) -> f64 {
        (self.distance_squared(other) as f64).sqrt()
    }
}

impl<const N: i32> From<PositionTuple> for ScaledPosition<N> {
    fn from(pos: PositionTuple) -> Self {
        Self { x: pos.0, y: pos.1 }
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
impl<const N: i32> Div<i32> for ScaledPosition<N> {
    type Output = Self;
    fn div(self, other: i32) -> Self::Output {
        Self::Output {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl<const N: i32> Sub<ScaledPosition<N>> for ScaledPosition<N> {
    type Output = Self;

    fn sub(self, other: ScaledPosition<N>) -> Self::Output {
        Self::Output {
            x: self.x - other.x,
            y: self.y - other.y,
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

impl<const N: i32> Add<ScaledPosition<N>> for ScaledPosition<N> {
    type Output = Self;

    fn add(self, other: ScaledPosition<N>) -> Self::Output {
        Self::Output {
            x: self.x + other.x,
            y: self.y + other.y,
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
