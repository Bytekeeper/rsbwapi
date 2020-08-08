use rsbwapi::*;

pub struct MyModule;

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
        let self_ = game.self_().unwrap();
        let names: Vec<String> = game
            .get_players()
            .iter()
            .map(|p| String::from(p.get_name()))
            .collect();
        for (i, name) in names.iter().enumerate() {
            game.cmd()
                .draw_text_screen((10, (i as i32) * 10 + 20), name);
        }
        game.cmd()
            .draw_text_screen((10, 10), game.enemy().unwrap().get_name());
        let units = game.get_all_units();
        let has_pool = units
            .iter()
            .any(|u| u.get_type() == UnitType::Zerg_Spawning_Pool);
        if let Some(u) = units
            .iter()
            .find(|u| u.get_type() == UnitType::Zerg_Hatchery)
        {
            if has_pool {
                u.train(UnitType::Zerg_Zergling)
            } else {
                u.train(UnitType::Zerg_Drone)
            }
        }
        let mineral = units
            .iter()
            .find(|u| u.get_type().is_mineral_field() && u.is_visible());
        let self_ = game.self_().unwrap();
        if self_.supply_used() == self_.supply_total() {
            if let Some(larva) = units.iter().find(|u| u.get_type() == UnitType::Zerg_Larva) {
                larva.train(UnitType::Zerg_Overlord)
            }
        }
        let builder = units
            .iter()
            .find(|u| u.get_type() == UnitType::Zerg_Drone)
            .unwrap();
        if self_.minerals() >= 200 && !has_pool {
            'outer: for y in 0..game.map_height() {
                for x in 0..game.map_width() {
                    if game.can_build_here(None, (x, y), UnitType::Zerg_Spawning_Pool, true) {
                        let tl = TilePosition { x, y }.to_position();
                        let br = tl + UnitType::Zerg_Spawning_Pool.tile_size().to_position();
                        game.cmd().draw_box_map(tl, br, Color::Red, false);
                        builder.build(UnitType::Zerg_Spawning_Pool, (x, y));
                        break 'outer;
                    }
                }
            }
        }

        if let Some(mineral) = mineral {
            units
                .iter()
                .filter(|u| {
                    u.get_type() == UnitType::Zerg_Drone
                        && !u.is_gathering_minerals()
                        && u != &builder
                })
                .for_each(|u| {
                    //                  println!("Sending {} to {}", u.id, mineral.id);
                    u.gather(mineral);
                });
            let enemy = units.iter().find(|u| u.get_player() == game.enemy());
            if let Some(enemy) = enemy {
                units
                    .iter()
                    .filter(|u| u.get_type() == UnitType::Zerg_Drone)
                    .for_each(|u| {
                        //                        println!("Sending {} to attack {:?}", u.id, enemy.get_type());
                        u.attack(enemy);
                    });
            }
        } else {
            println!("No minerals found!");
        }

        //        game.cmd().leave_game();

        for _bullet in game.get_bullets().iter() {
            /*            println!(
                "Bullet {} of player {:?} of unit {:?}",
                bullet.get_id(),
                bullet.get_player().map(|p| p.get_name().to_string()),
                bullet.get_source().map(|u| u.get_id())
            );
            */
        }
    }
}

fn main() {
    rsbwapi::start(MyModule);
}
