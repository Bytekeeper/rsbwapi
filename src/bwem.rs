use crate::{Rectangle, WalkPosition};

macro_rules! markable {
    ($m: ident, $n: ident) => {
        use std::sync::atomic::{AtomicI32, Ordering};
        static $m: AtomicI32 = AtomicI32::new(0);

        #[derive(Default)]
        pub(crate) struct $n {
            last_mark: i32,
        }

        impl $n {
            pub fn new() -> Self {
                Self { last_mark: 0 }
            }

            pub fn marked(&self) -> bool {
                $m.load(Ordering::Relaxed) == self.last_mark
            }

            pub fn set_marked(&mut self) {
                self.last_mark = $m.load(Ordering::Relaxed);
            }

            pub fn setUnmarked(&mut self) {
                self.last_mark = -1;
            }

            pub fn unmark_all() {
                $m.fetch_add(1, Ordering::Relaxed);
            }
        }
    };
}
pub mod area;
pub mod base;
pub mod cp;
pub mod defs;
pub mod graph;
pub mod map;
pub mod neutral;
pub mod tiles;

use crate::{Position, ScaledPosition, TilePosition};
fn outer_mini_tile_border(tl: WalkPosition, size: WalkPosition) -> Vec<WalkPosition> {
    Rectangle::new(tl - 1, tl + size + 1).border()
}

fn make_bounding_box_include_point<const N: i32>(
    top_left: &mut ScaledPosition<N>,
    bottom_right: &mut ScaledPosition<N>,
    a: ScaledPosition<N>,
) {
    if a.x < top_left.x {
        top_left.x = a.x
    }
    if a.x > bottom_right.x {
        bottom_right.x = a.x
    }

    if a.y < top_left.y {
        top_left.y = a.y
    }
    if a.y < bottom_right.y {
        bottom_right.y = a.y
    }
}

fn make_point_fit_to_bounding_box<const N: i32>(
    a: &mut ScaledPosition<N>,
    top_left: ScaledPosition<N>,
    bottom_right: ScaledPosition<N>,
) {
    if a.x < top_left.x {
        a.x = top_left.x
    } else if a.x > bottom_right.x {
        a.x = bottom_right.x
    }

    if a.y < top_left.y {
        a.y = top_left.y
    } else if a.y > bottom_right.y {
        a.y = bottom_right.y
    }
}

fn dist_to_rectangle(a: Position, top_left: TilePosition, size: TilePosition) -> i32 {
    let bottom_right = (top_left + size).to_position() - 1;
    let top_left = top_left.to_position();

    if a.x >= top_left.x {
        if a.x <= bottom_right.x {
            if a.y > bottom_right.y {
                a.y - bottom_right.y // S
            } else if a.y < top_left.y {
                top_left.y - a.y // N
            } else {
                0 // inside
            }
        } else {
            if a.y > bottom_right.y {
                rounded_dist(a, bottom_right) // SE
            } else if a.y < top_left.y {
                rounded_dist(a, Position::new(bottom_right.y, top_left.y))
            // NE
            } else {
                a.x - bottom_right.x // E
            }
        }
    } else if a.y > bottom_right.y {
        rounded_dist(a, Position::new(top_left.x, bottom_right.y)) // SW
    } else if a.y < top_left.y {
        rounded_dist(a, top_left) // NW
    } else {
        top_left.x - a.x // W
    }
}

const fn squared_norm(dx: i32, dy: i32) -> i32 {
    dx * dx + dy * dy
}

fn norm(dx: i32, dy: i32) -> f64 {
    (squared_norm(dx, dy) as f64).sqrt()
}

fn dist<const N: i32>(a: ScaledPosition<N>, b: ScaledPosition<N>) -> f64 {
    let c = b - a;
    norm(c.x, c.y)
}

fn rounded_dist<const N: i32>(a: ScaledPosition<N>, b: ScaledPosition<N>) -> i32 {
    (0.5 + dist(a, b)) as i32
}
