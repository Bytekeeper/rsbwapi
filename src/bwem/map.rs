use super::tiles::*;
use crate::*;

pub struct Map {
    size: isize,
    Size: TilePosition,
    walk_size: isize,
    Walk_size: WalkPosition,
    center: Position,
    tiles: Vec<Tile>,
    mini_tiles: Vec<MiniTile>,
}
