#[cfg(not(feature = "metrics"))]
macro_rules! measure {
    ($metric:expr, $e:expr) => {
        $e
    };
}

// mod bwem;
mod shm;
mod sma;

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
pub mod types;
pub mod unit;

pub use aimodule::AiModule;
pub use force::Force;
pub use game::Game;
pub use player::{Player, PlayerId};
pub use unit::{Unit, UnitId};

pub fn start(mut module: impl AiModule) {
    let mut client = client::Client::default();

    println!("Waiting for frame to start");

    while !client.get_game().is_in_game() {
        client.update(&mut module);
    }

    while client.get_game().is_in_game() {
        client.update(&mut module);
    }
}
