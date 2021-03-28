use core::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TilePosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WalkPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

pub type PositionTuple = (i32, i32);
pub const ORIGIN: Position = Position { x: 0, y: 0 };

impl Position {
    pub fn new(x: i32, y: i32) -> Option<Self> {
        Some(Self { x, y })
    }

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

impl Vector2D {
    pub fn new(x: f64, y: f64) -> Option<Self> {
        if x == 0.0 && y == 0.0 {
            None
        } else {
            Some(Self { x, y })
        }
    }
}

macro_rules! pos_math_ops {
    ($($t:ty)*) => ($(
        impl $t {
            pub fn is_valid(&self) -> bool {
                self.x >= 0 && self.y >= 0
            }

            pub fn distance_squared(&self, other: $t) -> u32 {
                let dx = self.x - other.x;
                let dy = self.y - other.y;
                (dx * dx + dy * dy) as u32
            }

            pub fn distance(&self, other: $t) -> f64 {
                (self.distance_squared(other) as f64).sqrt()
            }
        }

        impl From<PositionTuple> for $t {
            fn from(pos: PositionTuple) -> Self {
                Self { x: pos.0, y: pos.1 }
            }
        }

        impl From<$t> for PositionTuple {
            fn from(pos: $t) -> Self {
                (pos.x, pos.y)
            }
        }

        impl Mul<i32> for $t {
            type Output = $t;

            fn mul(self, other: i32) -> Self::Output {
                Self::Output {
                    x: self.x * other,
                    y: self.y * other
                }
            }
        }

        impl Mul<$t> for i32 {
            type Output = $t;

            fn mul(self, other: $t) -> Self::Output {
                Self::Output {
                     x: self * other.x,
                     y: self * other.y
                }
            }
        }

        impl Div<i32> for $t {
            type Output = Self;
            fn div(self, other: i32) -> Self::Output {
                Self::Output {
                    x: self.x / other,
                    y: self.y / other,
                }
            }
        }

        impl Sub<$t> for $t {
            type Output = Self;

            fn sub(self, other: $t) -> Self::Output {
                Self::Output {
                    x: self.x - other.x,
                    y: self.y - other.y,
                }
            }
        }

        impl Sub<PositionTuple> for $t {
            type Output = Self;

            fn sub(self, other: PositionTuple) -> Self::Output {
                Self::Output {
                    x: self.x - other.0,
                    y: self.y - other.1,
                }
            }
        }

        impl Add<$t> for $t {
            type Output = Self;

            fn add(self, other: $t) -> Self::Output {
                Self::Output {
                    x: self.x + other.x,
                    y: self.y + other.y,
                }
            }
        }

        impl Add<PositionTuple> for $t {
            type Output = Self;

            fn add(self, other: PositionTuple) -> Self::Output {
                Self::Output {
                    x: self.x + other.0,
                    y: self.y + other.1
                }
            }
        }
    )*)
}

pos_math_ops!(Position WalkPosition TilePosition);
