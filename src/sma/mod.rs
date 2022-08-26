use crate::*;
use ahash::AHashMap;
use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

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

#[derive(Debug, Clone)]
struct MiniTile {
    area_id: u16,
    mark: Cell<u16>,
    altitude: Altitude,
}

impl Default for MiniTile {
    fn default() -> Self {
        Self {
            area_id: 0,
            mark: Cell::new(0),
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

pub struct Base {
    pub position: TilePosition,
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
        let walk_grid = vec![true; scanw * (mapy + 2)];
        let resblock = vec![false; ((mapx + 2) * (mapy + 2)) as usize];
        let resval = vec![0; ((mapx + 2) * (mapy + 2)) as usize];
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
            for _ in p1.x..p2.x {
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

#[derive(Default)]
pub struct ChokePoint {
    index: usize,
    area_a: u16,
    area_b: u16,
    top: WalkPosition,
    walk_positions: Vec<WalkPosition>,
    mark: Cell<u16>,
    pred: Cell<usize>,
}

impl ChokePoint {
    fn new(
        index: usize,
        area_a: u16,
        area_b: u16,
        top: WalkPosition,
        walk_positions: Vec<WalkPosition>,
    ) -> Self {
        Self {
            index,
            top,
            walk_positions,
            area_a,
            area_b,
            mark: Cell::new(0),
            pred: Cell::new(0),
        }
    }
}

#[derive(Default)]
pub struct Map {
    mini_tiles: Vec<MiniTile>,
    mini_tile_mark: u16,
    walk_size: WalkPosition,
    pub bases: Vec<Base>,
    choke_points: Vec<ChokePoint>,
    distances: Vec<Vec<u32>>,
    paths: Vec<Vec<Vec<usize>>>,
}

impl Map {
    pub fn new(game: &Game) -> Self {
        let walk_size = WalkPosition::new(game.map_width() * 4, game.map_height() * 4);
        let mini_tiles = vec![MiniTile::default(); (walk_size.x * walk_size.y) as usize];
        let mut result = Self {
            mini_tiles,
            walk_size,
            ..Default::default()
        };
        result.assign_altitude_kind(game);
        result.find_bases(game);
        result.compute_altitude();
        result.assign_areas();
        result.area_paths(game);
        result.choke_point_paths();
        result
    }

    pub fn get_path(&self, from: WalkPosition, to: WalkPosition) -> (Vec<&ChokePoint>, u32) {
        let src_area = self.get_mini_tile(from).area_id;
        let target_area = self.get_mini_tile(to).area_id;
        let mut best: Option<((usize, usize), u32)> = None;
        if src_area != target_area {
            for (i, cp_a) in self
                .choke_points
                .iter()
                .enumerate()
                .filter(|(_, cp)| cp.area_a == src_area || cp.area_b == src_area)
            {
                for (j, cp_b) in self
                    .choke_points
                    .iter()
                    .enumerate()
                    .filter(|(_, cp)| cp.area_a == target_area || cp.area_b == target_area)
                {
                    let dist = from.distance(cp_a.top) as u32 * 8
                        + self.distances[i][j]
                        + to.distance(cp_b.top) as u32 * 8;
                    if let Some(tmp) = best {
                        if dist < tmp.1 {
                            best = Some(((i, j), dist));
                        }
                    } else {
                        best = Some(((i, j), dist));
                    }
                }
            }
        }
        best.map(|((i, j), dist)| {
            (
                self.paths[i][j]
                    .iter()
                    .map(|&i| &self.choke_points[i])
                    .collect(),
                dist,
            )
        })
        .unwrap_or_else(|| (vec![], from.distance(to) as u32))
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
                if let Altitude::Walkable(a) = self.get_mini_tile(wp).altitude {
                    walkpos_by_descending_altitude.push((wp, a));
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
        let mut frontiers = vec![];
        for (wp, _altitude) in walkpos_by_descending_altitude {
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

                        // Flood fill all tiles of b with id a
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

                        // Replace b with a in the frontiers
                        for ((i, j), _) in frontiers.iter_mut() {
                            if *i == b {
                                *i = a;
                            }
                            if *j == b {
                                *j = a;
                            }
                        }
                    } else {
                        if a > b {
                            std::mem::swap(&mut a, &mut b);
                        }
                        let counter = dist.entry((a, b)).or_insert(0);
                        self.get_mini_tile_mut(wp).area_id = if *counter % 2 == 0 { a } else { b };
                        *counter += 1;
                        frontiers.push(((a, b), wp));
                    }
                }
            }
        }

        struct Cluster {
            a: u16,
            b: u16,
            top: WalkPosition,
            wps: VecDeque<WalkPosition>,
        }
        let mut clusters = Vec::<Cluster>::new();
        for ((mut a, mut b), wp) in frontiers {
            // Part of merged border => continue
            if areas[a as usize].mini_tiles == 0 || areas[b as usize].mini_tiles == 0 {
                continue;
            }

            if a > b {
                std::mem::swap(&mut a, &mut b);
            }

            let cluster = clusters
                .iter_mut()
                .filter(|c| c.a == a && c.b == b)
                .map(|c| {
                    (
                        c.wps.front().unwrap().chebyshev_distance(wp),
                        c.wps.back().unwrap().chebyshev_distance(wp),
                        c,
                    )
                })
                .min_by_key(|(df, db, c)| *df.min(db))
                .filter(|(df, db, _)| df.min(db) < &20);
            if let Some((df, db, cluster)) = cluster {
                if df < db {
                    cluster.wps.push_front(wp);
                } else {
                    cluster.wps.push_back(wp);
                }
            } else {
                clusters.push(Cluster {
                    a,
                    b,
                    top: wp,
                    wps: VecDeque::from([wp]),
                });
            }
        }
        self.choke_points = clusters
            .drain(..)
            .enumerate()
            .map(|(i, c)| ChokePoint::new(i, c.a, c.b, c.top, c.wps.into()))
            .collect();
    }

    fn area_paths(&mut self, game: &Game) {
        #[derive(Eq, PartialEq)]
        struct Node {
            cost: u32,
            h: u32,
            pos: WalkPosition,
        }
        impl Ord for Node {
            fn cmp(&self, other: &Self) -> Ordering {
                (self.cost + self.h).cmp(&(other.cost + other.h)).reverse()
            }
        }
        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        self.paths = vec![vec![vec![]; self.choke_points.len()]; self.choke_points.len()];
        let mut distances = vec![vec![0; self.choke_points.len()]; self.choke_points.len()];
        let mut to_visit = BinaryHeap::new();
        for (i, cp) in self.choke_points.iter().enumerate() {
            let targets: Vec<_> = self
                .choke_points
                .iter()
                .enumerate()
                .skip(i + 1)
                .filter(|(_, t)| {
                    (t.area_a == cp.area_a
                        || t.area_a == cp.area_b
                        || t.area_b == cp.area_a
                        || t.area_b == cp.area_b)
                })
                .collect();
            for (j, t) in targets.iter() {
                to_visit.clear();
                to_visit.push(Node {
                    cost: 0,
                    h: 0,
                    pos: cp.top,
                });
                self.mini_tile_mark = self.mini_tile_mark.wrapping_add(1);
                while let Some(current) = to_visit.pop() {
                    if current.pos == t.top {
                        let distance = (current.cost as f32 * 8.0 / 10000.0 + 0.5) as u32;
                        distances[i][*j] = distance;
                        distances[*j][i] = distance;
                        break;
                    }
                    let mini_tile = self.get_mini_tile(current.pos);
                    if mini_tile.mark.get() == self.mini_tile_mark {
                        continue;
                    }
                    mini_tile.mark.set(self.mini_tile_mark);
                    for d in WALK_POSITION_8_DIR.iter() {
                        let next = current.pos + *d;
                        if game.is_valid(next) {
                            let diag_move = d.x != 0 && d.y != 0;
                            let add_cost = if diag_move { 14142 } else { 10000 };
                            to_visit.push(Node {
                                pos: next,
                                cost: current.cost + add_cost,
                                h: next.chebyshev_distance(t.top) * 10000,
                            });
                        }
                    }
                }
            }
        }
        self.distances = distances;
    }

    fn choke_point_paths(&mut self) {
        #[derive(Eq, PartialEq)]
        struct Node {
            cost: u32,
            h: u32,
            cp_index: usize,
            parent: usize,
        }
        impl Ord for Node {
            fn cmp(&self, other: &Self) -> Ordering {
                (self.cost + self.h).cmp(&(other.cost + other.h)).reverse()
            }
        }
        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        let mut to_visit = BinaryHeap::<Node>::new();
        let mut mark = 0;
        for (i, cp) in self.choke_points.iter().enumerate() {
            for (j, t) in self.choke_points.iter().enumerate().skip(i + 1) {
                if self.distances[i][j] != 0 {
                    continue;
                }
                mark += 1;
                to_visit.clear();
                to_visit.push(Node {
                    cost: 0,
                    h: 0,
                    cp_index: i,
                    parent: 0,
                });
                while let Some(current) = to_visit.pop() {
                    let current_cp = &self.choke_points[current.cp_index];
                    if current_cp.mark.get() == mark {
                        continue;
                    }

                    current_cp.mark.set(mark);
                    current_cp.pred.set(current.parent);
                    if current_cp.index == j {
                        self.distances[i][j] = current.cost;
                        self.distances[j][i] = current.cost;
                        let mut path = vec![j];
                        while let Some(prev) = path.last().cloned() {
                            let pred = self.choke_points[prev].pred.get();
                            path.push(pred);
                            if pred == i {
                                break;
                            }
                        }
                        self.paths[j][i] = path.clone();
                        path.reverse();
                        self.paths[i][j] = path;
                        break;
                    }

                    for (k, cp) in self.choke_points.iter().enumerate().filter(|(i, cp)| {
                        i != &current_cp.index
                            && (cp.area_a == current_cp.area_a
                                || cp.area_a == current_cp.area_b
                                || cp.area_b == current_cp.area_a
                                || cp.area_b == current_cp.area_b)
                    }) {
                        to_visit.push(Node {
                            cost: current.cost + self.distances[i][k],
                            h: current_cp.top.chebyshev_distance(current_cp.top) * 10000,
                            cp_index: k,
                            parent: current_cp.index,
                        });
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
    use imageproc::drawing::*;
    use inflate::inflate_bytes_zlib;
    use shm::Shm;
    use std::cell::RefCell;
    use std::fs::*;
    use std::path::Path;
    use std::time::Instant;

    #[test]
    fn test_maps() {
        let target = Path::new("target");
        for entry in read_dir("resources/test").unwrap().flatten()
        // .filter(|e| e.path().to_str().unwrap().contains(&"Andro"))
        {
            let mut target = target.to_path_buf();
            println!("Reading map {:?}", entry.path());
            let data = read(entry.path()).unwrap();
            let mut inflated = inflate_bytes_zlib(&data).unwrap();
            let shm = Shm::from_mut_slice(inflated.as_mut_slice().into());
            let mut game = Game::new(shm);
            let commands = RefCell::new(Commands::new());
            game.match_start();
            let timer = Instant::now();
            let tm = Map::new(&game);
            println!("{}", timer.elapsed().as_micros());
            let mut img = RgbImage::new(4 * game.map_width() as u32, 4 * game.map_height() as u32);
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
                        Altitude::Border => img.put_pixel(x as u32, y as u32, Rgb([255, 255, 255])),
                        _ => (),
                    }
                }
            }
            for base in game.get_start_locations() {
                let wp = base.to_walk_position();
                img.put_pixel(wp.x as u32, wp.y as u32, Rgb([255, 0, 0]));
                img.put_pixel(1 + wp.x as u32, wp.y as u32, Rgb([255, 0, 0]));
                img.put_pixel(1 + wp.x as u32, 1 + wp.y as u32, Rgb([255, 0, 0]));
                img.put_pixel(wp.x as u32, 1 + wp.y as u32, Rgb([255, 0, 0]));
            }
            for &Base { position: tp, .. } in &tm.bases {
                let wp = tp.to_walk_position();
                img.put_pixel(wp.x as u32, wp.y as u32, Rgb([255, 255, 255]));
                img.put_pixel(1 + wp.x as u32, wp.y as u32, Rgb([255, 255, 255]));
                img.put_pixel(1 + wp.x as u32, 1 + wp.y as u32, Rgb([255, 255, 255]));
                img.put_pixel(wp.x as u32, 1 + wp.y as u32, Rgb([255, 255, 255]));
            }
            for (i, targets) in tm.paths.iter().enumerate() {
                for (j, path) in targets.iter().enumerate().skip(i + 1) {
                    let mut p_i = path.iter();
                    if let Some(last) = p_i.next() {
                        let last = tm.choke_points[*last].top;
                        let mut last = (last.x as f32, last.y as f32);
                        for next in p_i {
                            let next = tm.choke_points[*next].top;
                            let next = (next.x as f32, next.y as f32);
                            draw_line_segment_mut(&mut img, last, next, Rgb([255, 255, 255]));
                            last = next;
                        }
                    }
                }
            }
            for ChokePoint {
                top,
                walk_positions,
                ..
            } in &tm.choke_points
            {
                for wp in walk_positions {
                    img.put_pixel(wp.x as u32, wp.y as u32, Rgb([255, 0, 0]));
                }
                img.put_pixel(top.x as u32, top.y as u32, Rgb([255, 255, 0]));
            }
            target.push(format!(
                "{}.png",
                entry.path().file_name().unwrap().to_string_lossy()
            ));
            eprintln!("{}", target.to_string_lossy());
            img.save(target).unwrap();
        }
        // panic!();
    }
}
