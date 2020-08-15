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
        /* Draw player names */
        let names: Vec<String> = game
            .get_players()
            .iter()
            .map(|p| String::from(p.get_name()))
            .collect();
        for (i, name) in names.iter().enumerate() {
            game.draw_text_screen((10, (i as i32) * 10 + 20), name);
        }
        game.draw_text_screen((10, 10), game.enemy().unwrap().get_name());
        let units = game.get_all_units();
        let my_units = self_.get_units();

        /* Draw BW Regions on all units */
        for u in units {
            let region = game.get_region_at(u.get_position());
            if region.is_none() {
                game.draw_text_map(u.get_position(), &"NO REGION".to_string());
            } else {
                game.draw_text_map(
                    u.get_position(),
                    &format!("r#{:?}", region.unwrap().get_id()),
                )
            }
        }
        /** Show larvas of Zerg depots 
        for hatchery in units.iter().filter(|u| u.get_type().produces_larva()) {
            println!(
                "Hatchery {} has {} larva",
                hatchery.get_id(),
                hatchery.get_larva().len()
            );
        }
         */
        let has_pool = my_units
            .iter()
            .any(|u| u.get_type() == UnitType::Zerg_Spawning_Pool);
        if let Some(u) = units
            .iter()
            .find(|u| u.get_type() == UnitType::Zerg_Hatchery)
        {
            if game.can_make(None, UnitType::Zerg_Zergling).unwrap_or(false) {
                u.train(UnitType::Zerg_Zergling).ok();
            } else {
                u.train(UnitType::Zerg_Drone).ok();
            }
        }
        let mineral = units
            .iter()
            .find(|u| u.get_type().is_mineral_field() && u.is_visible());
        let self_ = game.self_().unwrap();
        if self_.supply_used() == self_.supply_total() {
            if let Some(larva) = my_units.iter().find(|u| u.get_type() == UnitType::Zerg_Larva) {
                larva.train(UnitType::Zerg_Overlord).ok();
            }
        }
        let builder = if self_.minerals() >= UnitType::Zerg_Spawning_Pool.mineral_price()
            && !has_pool
        {
            let mut found = false;
            let builder = my_units
                .iter()
                .find(|u| u.get_type() == UnitType::Zerg_Drone)
                .expect("drone to build sth");
            'outer: for y in 0..game.map_height() {
                for x in 0..game.map_width() {
                    if game.can_build_here(builder, (x, y), UnitType::Zerg_Spawning_Pool, true).unwrap_or(false) {
                        let tl = TilePosition { x, y }.to_position();
                        let br = tl + UnitType::Zerg_Spawning_Pool.tile_size().to_position();
                        game.draw_box_map(tl, br, Color::Red, false);
                        builder.build(UnitType::Zerg_Spawning_Pool, (x, y)).ok();
                        found = true;
                        break 'outer;
                    }
                }
            }
            if found {
                Some(builder)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(mineral) = mineral {
            if let Some(miner) = game.get_closest_unit(
                mineral.get_position(),
                |u: &Unit| {
                    u.get_type() == UnitType::Zerg_Drone
                        && !u.is_gathering_minerals()
                        && Some(u) != builder
                },
                None,
            ) {
                miner.gather(mineral).ok();
            }

            let enemy = units.iter().find(|u| u.get_player() == game.enemy());
            if let Some(enemy) = enemy {
                units
                    .iter()
                    //                    .filter(|u| u.get_type() == UnitType::Zerg_Drone)
                    .for_each(|u| {
                        //                        println!("Sending {} to attack {:?}", u.id, enemy.get_type());
                        if let Err(err) = u.attack(enemy) {
                            println!("{:?} cannot hit {:?}: {:?}", u.get_type(), enemy.get_type(), err);
                        }
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
