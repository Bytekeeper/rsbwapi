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

impl From<PositionTuple> for WalkPosition {
    fn from(pos: PositionTuple) -> Self {
        Self { x: pos.0, y: pos.1 }
    }
}
