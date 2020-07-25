use rsbwapi::*;

use rsbwapi::types::{UnitType, UnitTypeExt};

pub struct MyModule {
    called: bool,
}

impl AiModule for MyModule {
    fn on_start(&self, game: &Game) {
        for location in game.get_start_locations() {
            println!("{:?}", location);
        }
    }

    fn on_unit_create(&self, _game: &Game, unit: Unit) {
        println!("Created Unit {}", unit.get_id())
    }

    fn on_unit_destroy(&self, _game: &Game, unit: Unit) {
        println!("Destroyed Unit {}", unit.get_id())
    }

    fn on_frame(&mut self, game: &Game) {
        let names: Vec<String> = game
            .get_players()
            .iter()
            .map(|p| String::from(p.name()))
            .collect();
        for (i, name) in names.iter().enumerate() {
            game.cmd().draw_text_screen((10, (i as i32) * 10 + 20), name);
        }
        game.cmd().draw_text_screen((10, 10), game.enemy().unwrap().name());
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
                        u.gather(mineral);
                    });
            } else {
                let enemy = units.iter().find(|u| u.get_player() == game.enemy());
                if let Some(enemy) = enemy {
                    units
                        .iter()
                        .filter(|u| u.get_type() == UnitType::Zerg_Drone)
                        .for_each(|u| {
                            println!("Sending {} to attack {:?}", u.id, enemy.get_type());
                            u.attack(enemy);
                        });
                }
            }
        } else {
            println!("No minerals found!");
        }

        for bullet in game.get_bullets().iter() {
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
    rsbwapi::start(MyModule { called: false });
}
