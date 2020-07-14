use types::UnitType;

mod shm;

pub mod aimodule;
mod bridge;
pub mod bullet;
pub mod client;
pub mod game;
pub mod player;
pub mod types;
pub mod unit;


pub use aimodule::AiModule;
pub use game::{Frame, Commands};
pub use unit::Unit;
pub use player::Player;

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
