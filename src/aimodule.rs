use crate::game::Game;
use crate::player::Player;
use crate::unit::Unit;
use bwapi_wrapper::prelude::Position;

pub trait AiModule {
    fn on_end(&mut self, _game: &Game, _winner: bool) {}
    fn on_nuke_detect(&mut self, _game: &Game, _position: Position) {}
    fn on_player_left(&mut self, _game: &Game, _player: Player) {}
    fn on_receive_text(&mut self, _game: &Game, _player: Player, _text: &str) {}
    fn on_save_game(&mut self, _game: &Game, _game_name: &str) {}
    fn on_send_text(&mut self, _game: &Game, _text: &str) {}
    fn on_start(&mut self, _game: &Game) {}
    fn on_frame(&mut self, state: &Game);
    fn on_unit_create(&mut self, _game: &Game, _unit: Unit) {}
    fn on_unit_destroy(&mut self, _game: &Game, _unit: Unit) {}
    fn on_unit_discover(&mut self, _game: &Game, _unit: Unit) {}
    fn on_unit_complete(&mut self, _game: &Game, _unit: Unit) {}
    fn on_unit_evade(&mut self, _game: &Game, _unit: Unit) {}
    fn on_unit_hide(&mut self, _game: &Game, _unit: Unit) {}
    fn on_unit_morph(&mut self, _game: &Game, _unit: Unit) {}
    fn on_unit_renegade(&mut self, _game: &Game, _unit: Unit) {}
    fn on_unit_show(&mut self, _game: &Game, _unit: Unit) {}
}
