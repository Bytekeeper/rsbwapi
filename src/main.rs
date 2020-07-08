use crate::aimodule::AiModule;
use crate::game::{Commands, Game};

mod shm;

pub mod aimodule;
mod bridge;
pub mod client;
pub mod game;
pub mod player;
pub mod types;
pub mod unit;

//use num_traits::{FromPrimitive, ToPrimitive};

use types::{UnitType, UnitTypeExt};


pub struct MyModule {
    called: bool,
}

impl AiModule for MyModule {
    fn on_start(&self, _game: &mut Game) {}

    fn on_frame(&mut self, game: &Game, cmd: &mut Commands) {
        let names: Vec<String> = game
            .get_players()
            .iter()
            .map(|p| String::from(p.name()))
            .collect();
        for (i, name) in names.iter().enumerate() {
            cmd.draw_text_screen((10, (i as i32) * 10 + 20), name);
        }
        cmd.draw_text_screen((10, 10), game.enemy().unwrap().name());
        let units = game.get_all_units();
        let mineral = units
            .iter()
            .find(|u| u.get_type().is_mineral_field() && u.is_visible(&game.self_().unwrap()));
        if let Some(mineral) = mineral {
            if !self.called {
                self.called = true;
                units
                    .iter()
                    .filter(|u| u.get_type() == UnitType::Zerg_Drone)
                    .for_each(|u| {
                        println!("Sending {} to {}", u.id, mineral.id);
                        cmd.issue_command(u.gather(mineral));
                    });
            } else {
                let enemy = units.iter().find(|u| u.get_player() == game.enemy());
                if let Some(enemy) = enemy {
                    units
                        .iter()
                        .filter(|u| u.get_type() == UnitType::Zerg_Drone)
                        .for_each(|u| {
                            println!("Sending {} to attack {:?}", u.id, enemy.get_type());
                            cmd.issue_command(u.attack(enemy));
                        });
                }
            }
        } else {
            println!("No minerals found!");
        }
    }
}

fn main() {
    let mut my_module = MyModule { called: false };
    let mut client = client::Client::default();

    println!("Waiting for game to start");

    while !client.get_game().is_in_game() {
        client.update(&mut my_module);
    }

    while client.get_game().is_in_game() {
        client.update(&mut my_module);
    }
}
