use core::ops::Div;
use core::ops::Mul;
use core::ops::Sub;

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

impl From<PositionTuple> for WalkPosition {
    fn from(pos: PositionTuple) -> Self {
        Self { x: pos.0, y: pos.1 }
    }
}

impl Mul<i32> for Position {
    type Output = Position;

    fn mul(self, other: i32) -> Self::Output {
        Self::Output {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Mul<Position> for i32 {
    type Output = Position;

    fn mul(self, other: Position) -> Self::Output {
        Self::Output {
            x: self * other.x,
            y: self * other.y,
        }
    }
}

impl Div<i32> for Position {
    type Output = Self;
    fn div(self, other: i32) -> Self::Output {
        Self::Output {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Sub<Position> for Position {
    type Output = Self;

    fn sub(self, other: Position) -> Self::Output {
        Self::Output {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
