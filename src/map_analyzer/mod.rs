use super::*;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Default, Copy, Clone)]
pub struct MiniTile {
    pub altitude: i32,
    pub area_id: i32,
}

pub struct TileMap {
    pub mini_tiles: Box<[[MiniTile; 256 * 4]]>,
    pub bases: Vec<TilePosition>,
}

#[derive(PartialEq, Eq)]
pub enum TileState {
    Invalid,
    Walkable,
    Unwalkable,
}

pub trait MapUtil {
    fn map_width(&self) -> i32;
    fn map_height(&self) -> i32;
    fn get_tile_state(&self, pos: &WalkPosition) -> TileState;
    fn get_static_minerals(&self) -> Vec<TilePosition>;
}

enum Neighbor {
    None,
    One(i32),
    Two(i32, i32),
}

impl<'a> MapUtil for Game<'a> {
    fn map_width(&self) -> i32 {
        self.map_width()
    }
    fn map_height(&self) -> i32 {
        self.map_height()
    }
    fn get_tile_state(&self, pos: &WalkPosition) -> TileState {
        if self.is_valid(pos) {
            if self.is_walkable(*pos) {
                TileState::Walkable
            } else {
                TileState::Unwalkable
            }
        } else {
            TileState::Invalid
        }
    }
    fn get_static_minerals(&self) -> Vec<TilePosition> {
        self.get_static_minerals()
            .iter()
            .map(|u| u.get_initial_tile_position())
            .collect()
    }
}

fn is_coast(map: &impl MapUtil, pos: WalkPosition) -> bool {
    map.get_tile_state(&pos) == TileState::Walkable
        && [
            WalkPosition::new(-1, 0),
            WalkPosition::new(0, -1),
            WalkPosition::new(1, 0),
            WalkPosition::new(0, 1),
        ]
        .iter()
        .any(|p| map.get_tile_state(&(*p + pos)) != TileState::Walkable)
}

impl TileMap {
    pub fn new(map: &impl MapUtil) -> Self {
        let mut result = Self {
            mini_tiles: vec![[MiniTile::default(); 256 * 4]; 256 * 4].into_boxed_slice(),
            bases: vec![],
        };
        result.create_bases(map);
        let mini_tiles_desc_by_altitude = result.compute_altitude(map);
        result.create_areas(map, mini_tiles_desc_by_altitude);
        result
    }

    fn create_bases(&mut self, map: &impl MapUtil) {
        struct Base {
            agg_x: i32,
            agg_y: i32,
            count: i32,
        }
        impl Base {
            fn center(&self) -> TilePosition {
                TilePosition::new(self.agg_x / self.count, self.agg_y / self.count)
            }
        }
        let mut pot_bases = vec![];
        for &mineral in &map.get_static_minerals() {
            let existing_base = pot_bases
                .iter_mut()
                .find(|b: &&mut Base| b.center().distance_squared(mineral) < 16 * 16);
            if let Some(base) = existing_base {
                base.agg_x += mineral.x;
                base.agg_y += mineral.y;
                base.count += 1;
            } else {
                pot_bases.push(Base {
                    agg_x: mineral.x,
                    agg_y: mineral.y,
                    count: 1,
                });
            }
        }
        println!(
            "{}: {:?}",
            pot_bases.len(),
            pot_bases
                .iter()
                .map(|it| format!("{}, {:?}", it.count, it.center()))
                .collect::<Vec<_>>()
        );
        self.bases = pot_bases.iter().map(|it| it.center()).collect();
    }

    fn create_areas(&mut self, map: &impl MapUtil, mini_tiles_desc_by_altitude: Vec<WalkPosition>) {
        #[derive(Debug)]
        struct AreaHolder {
            id: i32,
            wps: Vec<WalkPosition>,
            max_alt: i32,
        }

        impl AreaHolder {
            fn add(&mut self, wp: WalkPosition, tiles: &mut [[MiniTile; 256 * 4]]) {
                self.wps.push(wp);
                tiles[wp.y as usize][wp.x as usize].area_id = self.id;
            }
        }

        let mut areas = vec![AreaHolder {
            id: 0,
            wps: vec![],
            max_alt: 0,
        }];
        let mut border_count = HashMap::new();
        for &pos in &mini_tiles_desc_by_altitude {
            match self.find_adjacent_areas(map, pos) {
                Neighbor::None => {
                    let id = areas.len() as i32;
                    let mut area = AreaHolder {
                        id,
                        wps: vec![],
                        max_alt: self.mini_tiles[pos.y as usize][pos.x as usize].altitude,
                    };
                    area.add(pos, &mut *self.mini_tiles);
                    areas.push(area);
                }
                Neighbor::One(area_id) => {
                    areas[area_id as usize].add(pos, &mut *self.mini_tiles);
                }
                Neighbor::Two(smaller, bigger) => {
                    let (mut smaller, mut bigger): (usize, usize) =
                        (smaller as usize, bigger as usize);
                    if areas[smaller as usize].wps.len() > areas[bigger as usize].wps.len() {
                        std::mem::swap(&mut smaller, &mut bigger);
                    }
                    assert_ne!(smaller, bigger);
                    let (mut a, mut b) = (smaller, bigger);
                    if a > b {
                        std::mem::swap(&mut a, &mut b);
                    }
                    let current_alt = self.mini_tiles[pos.y as usize][pos.x as usize].altitude;
                    if areas[smaller].wps.len() < 80
                        || areas[smaller].max_alt < 10
                        || current_alt * 10 > areas[bigger].max_alt * 9
                        || current_alt * 10 > areas[smaller].max_alt * 9
                    {
                        areas[bigger].wps.push(pos);
                        for pos in &areas[smaller].wps {
                            self.mini_tiles[pos.y as usize][pos.x as usize].area_id = bigger as i32;
                        }
                        let mut tmp = std::mem::take(&mut areas[smaller].wps);
                        areas[bigger].wps.append(&mut tmp);
                        areas[bigger].add(pos, &mut *self.mini_tiles);
                    } else {
                        let value = border_count.entry((a, b)).or_insert(0);
                        let chosen = if *value % 2 == 0 { a } else { b };
                        *value += 1;
                        areas[chosen].wps.push(pos);
                        areas[chosen].add(pos, &mut *self.mini_tiles);
                    }
                }
            }
        }
    }

    fn find_adjacent_areas(&self, map: &impl MapUtil, pos: WalkPosition) -> Neighbor {
        let mut result = Neighbor::None;
        for wp in &[
            WalkPosition::new(-1, 0),
            WalkPosition::new(0, -1),
            WalkPosition::new(1, 0),
            WalkPosition::new(0, 1),
        ] {
            let pos = *wp + pos;
            if map.get_tile_state(&pos) != TileState::Invalid {
                let area_id = self.mini_tiles[pos.y as usize][pos.x as usize].area_id;
                if area_id > 0 {
                    result = match result {
                        Neighbor::None => Neighbor::One(area_id),
                        Neighbor::One(x) if x != area_id => Neighbor::Two(x, area_id),
                        Neighbor::Two(x, y) if x != area_id && area_id < y => {
                            Neighbor::Two(x, area_id)
                        }
                        rem => rem,
                    };
                }
            }
        }
        result
    }

    fn compute_altitude(&mut self, map: &impl MapUtil) -> Vec<WalkPosition> {
        #[derive(Eq, PartialEq, Debug)]
        struct Altitude {
            pos: WalkPosition,
            altitude: i32,
        }
        // Horizon ordered by |altitude|
        impl PartialOrd for Altitude {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for Altitude {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.altitude
                    .abs()
                    .cmp(&other.altitude.abs())
                    .then_with(|| self.pos.x.cmp(&other.pos.x))
                    .then_with(|| self.pos.y.cmp(&other.pos.y))
                    .reverse()
            }
        }

        // Instead of second pass, we get a list of mini tiles sorted by altitude almost free here
        let mut mini_tiles_sorted_by_altitude = vec![];
        let mut horizon = BinaryHeap::new();
        for y in 0..map.map_height() * 4 {
            for x in 0..map.map_width() * 4 {
                let wp = WalkPosition::new(x, y);
                if is_coast(map, wp) {
                    horizon.push(Altitude {
                        pos: wp,
                        altitude: 0,
                    });
                    mini_tiles_sorted_by_altitude.push(wp);
                }
            }
        }
        while !horizon.is_empty() {
            let Altitude { pos, altitude } = horizon.pop().unwrap();
            let (x, y) = (pos.x as usize, pos.y as usize);
            if self.mini_tiles[y][x].altitude == 0 {
                let mut altitude = altitude;
                match map.get_tile_state(&pos) {
                    TileState::Walkable => {
                        altitude = 1.max(altitude + 1);
                        self.mini_tiles[y][x].altitude = altitude;
                        mini_tiles_sorted_by_altitude.push(pos);
                    }
                    TileState::Unwalkable => {
                        altitude = (-1).min(altitude - 1);
                        self.mini_tiles[y][x].altitude = altitude;
                    }
                    _ => panic!("Should not traverse invalid minitiles"),
                }
                for &p in &[
                    WalkPosition::new(-1, 0),
                    WalkPosition::new(-1, -1),
                    WalkPosition::new(0, -1),
                    WalkPosition::new(1, -1),
                    WalkPosition::new(1, 0),
                    WalkPosition::new(1, 1),
                    WalkPosition::new(0, 1),
                    WalkPosition::new(-1, 1),
                ] {
                    let pos = pos + p;
                    if map.get_tile_state(&pos) != TileState::Invalid {
                        horizon.push(Altitude { pos, altitude });
                    }
                }
            }
        }
        mini_tiles_sorted_by_altitude.reverse();
        mini_tiles_sorted_by_altitude
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{command::Commands, game::*};
    use image::*;
    use inflate::inflate_bytes_zlib;
    use shm::Shm;
    use std::cell::RefCell;
    use std::fs::*;
    use std::time::Instant;

    #[test]
    fn test_maps() {
        for entry in read_dir("resources/test").unwrap() {
            let entry = entry.unwrap();
            println!("Reading map {:?}", entry.path());
            let data = read(entry.path()).unwrap();
            let mut inflated = inflate_bytes_zlib(&data).unwrap();
            let shm = unsafe { Shm::from_slice(inflated.as_mut_slice().into()) };
            let mut game_context = GameContext::new(shm);
            let commands = RefCell::new(Commands::new());
            game_context.match_start();
            game_context.with_frame(&commands, |game| {
                let timer = Instant::now();
                let tm = TileMap::new(game);
                println!("{}", timer.elapsed().as_micros());
                let mut img: RgbImage =
                    ImageBuffer::new(4 * game.map_width() as u32, 4 * game.map_height() as u32);
                for (y, row) in tm
                    .mini_tiles
                    .iter()
                    .enumerate()
                    .take(game.map_height() as usize * 4)
                {
                    for (x, alt) in row.iter().enumerate().take(game.map_width() as usize * 4) {
                        let a = alt.altitude;
                        if a < 0 {
                            img.put_pixel(x as u32, y as u32, Rgb([0, 0, 255 - 3 * (-a) as u8]));
                        } else if alt.area_id > 0 {
                            img.put_pixel(
                                x as u32,
                                y as u32,
                                Rgb([255 - 4 * a as u8, (37 * alt.area_id % 255) as u8, 0]),
                            );
                        }
                    }
                }
                for &tp in &tm.bases {
                    let wp = tp.to_walk_position();
                    img.put_pixel(wp.x as u32, wp.y as u32, Rgb([255, 255, 255]));
                    img.put_pixel(1 + wp.x as u32, wp.y as u32, Rgb([255, 255, 255]));
                    img.put_pixel(1 + wp.x as u32, 1 + wp.y as u32, Rgb([255, 255, 255]));
                    img.put_pixel(wp.x as u32, 1 + wp.y as u32, Rgb([255, 255, 255]));
                }
                img.save(format!(
                    "{}.png",
                    entry.path().file_name().unwrap().to_string_lossy()
                ))
                .unwrap();
            });
        }
        panic!();
    }
}
