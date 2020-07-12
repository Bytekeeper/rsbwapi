use crate::aimodule::AiModule;
use crate::game::{Commands, Frame};
use crate::unit::Unit;

mod shm;

pub mod aimodule;
mod bridge;
pub mod bullet;
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
    fn on_start(&self, frame: &Frame) {
        for location in frame.get_start_locations() {
            println!("{:?}", location);
        }
    }

    fn on_unit_create(&self, _frame: &Frame, _cmd: &mut Commands, unit: Unit) {
        println!("Created Unit {}", unit.get_id())
    }

    fn on_unit_destroy(&self, _frame: &Frame, _cmd: &mut Commands, unit: Unit) {
        println!("Destroyed Unit {}", unit.get_id())
    }

    fn on_frame(&mut self, frame: &Frame, cmd: &mut Commands) {
        let names: Vec<String> = frame
            .get_players()
            .iter()
            .map(|p| String::from(p.name()))
            .collect();
        for (i, name) in names.iter().enumerate() {
            cmd.draw_text_screen((10, (i as i32) * 10 + 20), name);
        }
        cmd.draw_text_screen((10, 10), frame.enemy().unwrap().name());
        let units = frame.get_all_units();
        let mineral = units
            .iter()
            .find(|u| u.get_type().is_mineral_field() && u.is_visible(&frame.self_().unwrap()));
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
                let enemy = units.iter().find(|u| u.get_player() == frame.enemy());
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

        for bullet in frame.get_bullets().iter() {
            println!(
                "Bullet {} of player {:?} of unit {:?}",
                bullet.get_id(),
                bullet.get_player().map(|p| p.name().to_string()),
                bullet.get_source().map(|u| u.get_id())
            );
        }
    }
}

fn main() {
    let mut my_module = MyModule { called: false };
    let mut client = client::Client::default();

    println!("Waiting for frame to start");

    while !client.get_game().is_in_game() {
        client.update(&mut my_module);
    }

    while client.get_game().is_in_game() {
        client.update(&mut my_module);
    }
}
