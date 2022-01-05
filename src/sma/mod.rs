use crate::*;
use ahash::AHashMap;

const MINERAL_MIN: i32 = 500;
const BASE_MIN: i32 = 400;
const INC_DIST: i32 = 9 * 32;
const SQRT_ARR: [i32; 64] = [
    0, 300, 150, 100, 75, 60, 50, 42, 300, 212, 134, 94, 72, 58, 49, 42, 150, 134, 106, 83, 67, 55,
    47, 41, 100, 94, 83, 70, 60, 51, 44, 39, 75, 72, 67, 60, 53, 46, 41, 37, 60, 58, 55, 51, 46,
    42, 38, 34, 50, 49, 47, 44, 41, 38, 35, 32, 42, 42, 41, 39, 37, 34, 32, 30,
];

// first 24 entries should be unused. formula sqrt(900/(x*x+y*y)), 3,0 gives val 10
const fn squared_norm(dx: i32, dy: i32) -> i32 {
    dx * dx + dy * dy
}

fn norm(dx: i32, dy: i32) -> f32 {
    (squared_norm(dx, dy) as f32).sqrt()
}

#[derive(Debug, Clone, Copy)]
enum Altitude {
    Invalid,
    Border,
    Walkable(i16),
    Unwalkable(i16),
    Hole,
}

#[derive(Debug, Clone, Copy)]
struct MiniTile {
    area_id: u16,
    altitude: Altitude,
}

impl Default for MiniTile {
    fn default() -> Self {
        Self {
            area_id: 0,
            altitude: Altitude::Invalid,
        }
    }
}

impl MiniTile {
    fn is_altitude_missing(&self) -> bool {
        matches!(
            self.altitude,
            Altitude::Walkable(0) | Altitude::Unwalkable(0)
        )
    }

    fn is_walkable(&self) -> bool {
        matches!(self.altitude, Altitude::Walkable(_))
    }

    fn is_border(&self) -> bool {
        matches!(self.altitude, Altitude::Border)
    }

    fn set_altitude(&mut self, altitude: i16) {
        match self.altitude {
            Altitude::Walkable(_) => self.altitude = Altitude::Walkable(altitude),
            Altitude::Unwalkable(_) => self.altitude = Altitude::Unwalkable(altitude),
            _ => panic!(),
        }
    }

    fn set_border(&mut self) {
        self.altitude = Altitude::Border;
    }
}

struct Base {
    position: TilePosition,
    minerals: Vec<UnitId>,
    geysers: Vec<UnitId>,
}

// JAJ's Base finder
struct BaseFinder {
    mapx: usize,
    mapy: usize,
    scanw: usize,
    walk_grid: Vec<bool>,
    resblock: Vec<bool>,
    resval: Vec<i32>,
}

impl BaseFinder {
    fn find(game: &Game) -> Vec<Base> {
        let mapx = game.map_width() as usize;
        let mapy = game.map_height() as usize;
        let scanw = mapx + 2;
        let mut walk_grid = vec![true; scanw * (mapy + 2)];
        let mut resblock = vec![false; ((mapx + 2) * (mapy + 2)) as usize];
        let mut resval = vec![0; ((mapx + 2) * (mapy + 2)) as usize];
        let mut finder = BaseFinder {
            mapx,
            mapy,
            scanw,
            walk_grid,
            resblock,
            resval,
        };
        let mut curoff = mapx + 3; // 1,1 in walkgrid = 0,0 in bwapi
        for y in 0..mapy {
            for x in 0..mapx {
                finder.walk_grid[curoff] = false;
                for ym in 0..4 {
                    for xm in 0..4 {
                        if !game.is_walkable(WalkPosition::new(
                            (x * 4 + xm) as i32,
                            (y * 4 + ym) as i32,
                        )) {
                            finder.walk_grid[curoff] = true;
                            break;
                        }
                    }
                }
                curoff += 1;
            }
            curoff += 2;
        }

        let mut res: Vec<Unit> = Vec::with_capacity(200);

        for u in game.get_static_minerals() {
            if u.get_initial_resources() < MINERAL_MIN {
                continue;
            }
            let p = u.get_initial_tile_position();
            finder.mark_res_block(p, 2, 1);
            finder.mark_border_value(p, 2, 1, 1);
            res.push(u);
        }
        for u in game.get_static_geysers() {
            if u.get_initial_resources() == 0 {
                continue;
            }
            let p = u.get_initial_tile_position();
            finder.mark_res_block(p, 4, 1);
            finder.mark_border_value(p, 4, 2, 3);
            res.push(u);
        }

        let mut potbase = Vec::with_capacity((scanw * mapy / 8) as usize);
        for off in scanw..scanw * (mapy + 1) {
            if finder.resval[off] > BASE_MIN && !finder.resblock[off] {
                potbase.push(off)
            }
        }
        potbase.sort_by_key(|&b| std::cmp::Reverse(finder.resval[b]));

        let mut bases = vec![];

        for off in potbase {
            if finder.resval[off] <= BASE_MIN || finder.resblock[off] {
                continue;
            }

            let mut base = Base {
                position: TilePosition::new(
                    ((off - mapx - 3) % scanw) as i32,
                    ((off - mapx - 3) / scanw) as i32,
                ),
                minerals: vec![],
                geysers: vec![],
            };

            let bp = base.position.to_position() + (64, 48);

            let mut i = 0;
            while i < res.len() {
                let diff = bp - res[i].get_initial_position();
                if diff.x * diff.x + diff.y * diff.y > INC_DIST * INC_DIST {
                    i += 1;
                    continue;
                }

                if res[i].get_initial_type().is_mineral_field() {
                    // base.minerals.push(res[i]);
                    base.minerals.push(res[i].get_id());
                    finder.mark_border_value(res[i].get_initial_tile_position(), 2, 1, -1);
                } else {
                    // base.geysers.push(res[i]);
                    base.geysers.push(res[i].get_id());
                    finder.mark_border_value(res[i].get_initial_tile_position(), 4, 2, -3);
                }
                res.swap_remove(i);
            }
            bases.push(base);
        }
        bases
    }

    fn tile_off(&self, x: usize, y: usize) -> usize {
        x + 1 + (y + 1) * (self.mapx + 2)
    }
    fn mark_res_block(&mut self, p: TilePosition, tw: i32, th: i32) {
        let p1 = TilePosition::new(0.max(p.x - 6), 0.max(p.y - 5));
        let p2 = TilePosition::new(
            (self.mapx as i32 - 1).min(p.x + 2 + tw),
            (self.mapy as i32 - 1).min(p.y + 2 + th),
        );

        for y in p1.y..p2.y {
            let mut off = self.tile_off(p1.x as usize, y as usize);
            for x in p1.x..p2.x {
                self.resblock[off] = true;
                off += 1;
            }
        }
    }
    fn mark_row(
        &mut self,
        midoff: usize,
        distrow: usize,
        mid: usize,
        end: usize,
        inc: usize,
        valmod: i32,
    ) -> bool {
        let mut writes = 0;
        let mut roff = midoff + inc;
        for i in 1..=end {
            if self.walk_grid[roff] {
                break;
            }
            self.resval[roff] += valmod * SQRT_ARR[distrow + i];
            roff += inc;
            writes += 1;
        }
        roff = midoff;
        for i in 1 - mid as isize..=end as isize {
            if self.walk_grid[roff] {
                break;
            }
            self.resval[roff] += valmod * SQRT_ARR[distrow + i.max(0) as usize];
            roff -= inc;
            writes += 1;
        }
        writes > 0
    }

    fn mark_border_value(&mut self, p: TilePosition, tw: i32, th: i32, valmod: i32) {
        let coff = self.tile_off((p.x + tw - 1) as usize, (p.y + th - 1) as usize);

        let mut c = false;
        for i in th..th + 6 {
            if self.walk_grid[coff - i as usize * self.scanw] {
                c = true;
                break;
            }
        }
        if !c {
            for s in 3..7 {
                if !self.mark_row(
                    coff - (s + 2 + th) as usize * self.scanw,
                    s as usize * 8,
                    (tw + 3) as usize,
                    s as usize,
                    1,
                    valmod,
                ) {
                    break;
                }
            }
        }
        c = false;
        for i in 1..5 {
            if self.walk_grid[coff + i as usize * self.scanw] {
                c = true;
                break;
            }
        }
        if !c {
            for s in 3..7 {
                if !self.mark_row(
                    coff + (s + 1) as usize * self.scanw,
                    s as usize * 8,
                    (tw + 3) as usize,
                    s as usize,
                    1,
                    valmod,
                ) {
                    break;
                }
            }
        }
        c = false;
        for i in tw..tw + 7 {
            if self.walk_grid[coff - i as usize] {
                c = true;
                break;
            }
        }
        if !c {
            for s in 3..7 {
                if !self.mark_row(
                    coff - (s + 3 + tw) as usize,
                    s as usize * 8,
                    (th + 2) as usize,
                    1 + s as usize,
                    self.scanw,
                    valmod,
                ) {
                    break;
                }
            }
        }
        c = false;
        for i in 1..5 {
            if self.walk_grid[coff + i as usize] {
                c = true;
                break;
            }
        }
        if !c {
            for s in 3..7 {
                if !self.mark_row(
                    coff + (s + 1) as usize,
                    s as usize * 8,
                    (th + 2) as usize,
                    1 + s as usize,
                    self.scanw,
                    valmod,
                ) {
                    break;
                }
            }
        }
    }
}

struct Map {
    mini_tiles: Vec<MiniTile>,
    walk_size: WalkPosition,
    bases: Vec<Base>,
}

impl Map {
    fn new(game: &Game) -> Self {
        let walk_size = WalkPosition::new(game.map_width() * 4, game.map_height() * 4);
        let mini_tiles = vec![MiniTile::default(); (walk_size.x * walk_size.y) as usize];
        let mut result = Self {
            mini_tiles,
            walk_size,
            bases: vec![],
        };
        result.assign_altitude_kind(game);
        result.find_bases(game);
        result.compute_altitude();
        result.assign_areas();
        result
    }

    fn find_bases(&mut self, game: &Game) {
        self.bases = BaseFinder::find(game);
    }

    fn assign_altitude_kind(&mut self, game: &Game) {
        let is_border = |w: WalkPosition| {
            dir_4()
                .iter()
                .any(|&d| game.is_valid(w + d) && game.is_walkable(w + d))
        };
        let mut visited = vec![];
        let mut horizon = vec![];
        for y in 0..self.walk_size.y {
            for x in 0..self.walk_size.x {
                let wp = WalkPosition::new(x, y);
                if matches!(self.get_mini_tile(wp).altitude, Altitude::Invalid) {
                    if game.is_walkable(wp) {
                        self.get_mini_tile_mut(wp).altitude = Altitude::Walkable(0);
                    } else if is_border(wp) {
                        visited.clear();
                        horizon.clear();
                        horizon.push(wp);
                        while visited.len() < 200 {
                            if let Some(wp) = horizon.pop() {
                                if !visited.contains(&wp) {
                                    visited.push(wp);
                                    for d in dir_4() {
                                        if self.valid(wp + d) && !game.is_walkable(wp + d) {
                                            horizon.push(wp + d);
                                        }
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                        if visited.len() < 200 {
                            for &wp in &visited {
                                self.get_mini_tile_mut(wp).altitude = Altitude::Hole;
                            }
                        } else {
                            for &wp in &visited {
                                self.get_mini_tile_mut(wp).altitude = if is_border(wp) {
                                    Altitude::Border
                                } else {
                                    Altitude::Unwalkable(0)
                                };
                            }
                        }
                    }
                }
            }
        }
    }

    fn valid(&self, p: WalkPosition) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.walk_size.x && p.y < self.walk_size.y
    }

    fn get_mini_tile(&self, p: WalkPosition) -> &MiniTile {
        &self.mini_tiles[(p.y * self.walk_size.x + p.x) as usize]
    }

    fn get_mini_tile_mut(&mut self, p: WalkPosition) -> &mut MiniTile {
        &mut self.mini_tiles[(p.y * self.walk_size.x + p.x) as usize]
    }

    fn compute_altitude(&mut self) {
        let range = self.walk_size.x.max(self.walk_size.y) / 2 + 3;
        let mut deltas_by_ascending_altitude: Vec<_> = (0..=range)
            .flat_map(|dy| (dy..=range).map(move |dx| (dx, dy)))
            .filter(|&(dx, dy)| dx != 0 || dy != 0)
            .map(|(dx, dy)| (WalkPosition::new(dx, dy), (0.5 + norm(dx, dy) * 8.0) as i16))
            .collect();
        deltas_by_ascending_altitude.sort_by_key(|(_, a)| *a);

        let mut border = vec![];

        for y in -1..=self.walk_size.y {
            for x in -1..=self.walk_size.x {
                let w = WalkPosition::new(x, y);
                if !self.valid(w) || self.get_mini_tile_mut(w).is_border() {
                    border.push((w, 0));
                }
            }
        }

        for (d, altitude) in deltas_by_ascending_altitude {
            let mut i = 0;
            while i < border.len() {
                let current = &mut border[i];
                if altitude - current.1 >= 2 * 8 {
                    border.swap_remove(i);
                } else {
                    for delta in [
                        WalkPosition::new(d.x, d.y),
                        WalkPosition::new(-d.x, d.y),
                        WalkPosition::new(d.x, -d.y),
                        WalkPosition::new(-d.x, -d.y),
                        WalkPosition::new(d.y, d.x),
                        WalkPosition::new(-d.y, d.x),
                        WalkPosition::new(d.y, -d.x),
                        WalkPosition::new(-d.y, -d.x),
                    ] {
                        let w = current.0 + delta;
                        if self.valid(w) {
                            let mini_tile = self.get_mini_tile_mut(w);
                            if mini_tile.is_altitude_missing() {
                                mini_tile.set_altitude(altitude);
                                current.1 = altitude;
                            }
                        }
                    }
                    i += 1;
                }
            }
        }
    }

    fn assign_areas(&mut self) {
        let mut walkpos_by_descending_altitude = Vec::with_capacity(self.mini_tiles.len());
        for y in 0..self.walk_size.y {
            for x in 0..self.walk_size.x {
                let wp = WalkPosition::new(x, y);
                match self.get_mini_tile(wp).altitude {
                    Altitude::Walkable(a) => walkpos_by_descending_altitude.push((wp, a)),
                    _ => (),
                }
            }
        }
        walkpos_by_descending_altitude.sort_by_key(|(_, a)| std::cmp::Reverse(*a));

        enum Neighbors {
            None,
            One(u16),
            Two(u16, u16),
        }
        let mut dist = AHashMap::new();

        #[derive(Default)]
        struct Area {
            mini_tiles: u32,
        }
        let mut areas = vec![Area::default()];
        let mut horizon = vec![];
        for (wp, altitude) in walkpos_by_descending_altitude {
            let mut n = Neighbors::None;
            for &d in &WALK_POSITION_4_DIR {
                if self.valid(wp + d) {
                    let n_id = self.get_mini_tile(wp + d).area_id;
                    if n_id > 0 {
                        n = match n {
                            Neighbors::None => Neighbors::One(n_id),
                            Neighbors::One(a) if a != n_id => Neighbors::Two(a, n_id),
                            Neighbors::Two(a, b) if a != n_id && n_id < b => {
                                Neighbors::Two(a, n_id)
                            }
                            x => x,
                        };
                    }
                }
            }
            match n {
                Neighbors::None => {
                    self.get_mini_tile_mut(wp).area_id = areas.len() as u16;
                    areas.push(Area { mini_tiles: 1 });
                }
                Neighbors::One(id) => {
                    self.get_mini_tile_mut(wp).area_id = id;
                    areas[id as usize].mini_tiles += 1;
                }
                Neighbors::Two(mut a, mut b) => {
                    if areas[a as usize].mini_tiles < 400
                        || areas[b as usize].mini_tiles < 400
                        || self
                            .bases
                            .iter()
                            .any(|b| b.position.distance(wp.to_tile_position()) < 5.0)
                    {
                        areas[a as usize].mini_tiles += areas[b as usize].mini_tiles;
                        areas[b as usize].mini_tiles = 0;
                        horizon.clear();
                        horizon.push(wp);
                        while let Some(wp) = horizon.pop() {
                            self.get_mini_tile_mut(wp).area_id = a;
                            for &d in &WALK_POSITION_4_DIR {
                                let next = wp + d;
                                if self.valid(next) && self.get_mini_tile(next).area_id == b {
                                    horizon.push(next);
                                }
                            }
                        }
                    } else {
                        if a > b {
                            std::mem::swap(&mut a, &mut b);
                        }
                        let counter = dist.entry((a, b)).or_insert(0);
                        self.get_mini_tile_mut(wp).area_id = if *counter % 2 == 0 { a } else { b };
                        *counter += 1;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use super::*;
    use crate::{command::Commands, game::*};
    use image::*;
    use inflate::inflate_bytes_zlib;
    use shm::Shm;
    use std::cell::RefCell;
    use std::fs::*;
    use std::path::Path;
    use std::time::Instant;

    #[test]
    fn test_maps() {
        let target = Path::new("target");
        for entry in read_dir("resources/test")
            .unwrap()
            .flatten()
            .filter(|e| e.path().to_str().unwrap().contains(&"Andro"))
        {
            let mut target = target.to_path_buf();
            println!("Reading map {:?}", entry.path());
            let data = read(entry.path()).unwrap();
            let mut inflated = inflate_bytes_zlib(&data).unwrap();
            let shm = Shm::from_mut_slice(inflated.as_mut_slice().into());
            let mut game_context = GameContext::new(shm);
            let commands = RefCell::new(Commands::new());
            game_context.match_start();
            game_context.with_frame(&commands, |game| {
                let timer = Instant::now();
                let tm = Map::new(game);
                println!("{}", timer.elapsed().as_micros());
                let mut img: RgbImage =
                    ImageBuffer::new(4 * game.map_width() as u32, 4 * game.map_height() as u32);
                for y in 0..tm.walk_size.y {
                    for x in 0..tm.walk_size.x {
                        let alt = tm.get_mini_tile(WalkPosition::new(x, y));
                        match alt.altitude {
                            Altitude::Unwalkable(a) => {
                                img.put_pixel(x as u32, y as u32, Rgb([0, 0, 255 - a as u8]))
                            }
                            Altitude::Walkable(a) => img.put_pixel(
                                x as u32,
                                y as u32,
                                Rgb([255 - (a / 2) as u8, (27 * alt.area_id % 255) as u8, 255]),
                            ),
                            Altitude::Border => {
                                img.put_pixel(x as u32, y as u32, Rgb([255, 255, 255]))
                            }
                            _ => (),
                        }
                    }
                }
                for &Base { position: tp, .. } in &tm.bases {
                    let wp = tp.to_walk_position();
                    img.put_pixel(wp.x as u32, wp.y as u32, Rgb([255, 255, 255]));
                    img.put_pixel(1 + wp.x as u32, wp.y as u32, Rgb([255, 255, 255]));
                    img.put_pixel(1 + wp.x as u32, 1 + wp.y as u32, Rgb([255, 255, 255]));
                    img.put_pixel(wp.x as u32, 1 + wp.y as u32, Rgb([255, 255, 255]));
                }
                target.push(format!(
                    "{}.png",
                    entry.path().file_name().unwrap().to_string_lossy()
                ));
                img.save(target).unwrap();
            });
            break;
        }
        panic!();
    }
}
