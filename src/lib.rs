#[cfg(not(feature = "metrics"))]
macro_rules! measure {
    ($metric:expr, $e:expr) => {
        $e
    };
}

// mod bwem;
mod projected;
mod shm;

pub use crate::types::*;
pub use bwapi_wrapper::prelude::*;
pub mod aimodule;
pub mod bullet;
pub mod can_do;
pub mod client;
pub mod command;
pub mod force;
pub mod game;
pub mod player;
pub mod predicate;
pub mod region;
pub mod sma;
pub mod types;
pub mod unit;

pub use aimodule::AiModule;
pub use bullet::{Bullet, BulletType};
pub use force::Force;
pub use game::Game;
pub use player::{Player, PlayerId};
pub use unit::{Unit, UnitId};

pub fn start<M: AiModule>(build_module: impl FnOnce(&Game) -> M) {
    let mut client = client::Client::default();

    println!("Waiting for frame to start");
    let mut module = Box::new(build_module(client.get_game()));

    while !client.get_game().is_in_game() {
        client.update(&mut *module);
    }

    while client.get_game().is_in_game() {
        client.update(&mut *module);
    }
}
