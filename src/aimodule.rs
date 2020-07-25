use crate::game::Game;
use crate::player::Player;
use crate::position::Position;
use crate::unit::Unit;

pub trait AiModule {
    fn on_end(&self, _game: &Game, _winner: bool) {}
    fn on_nuke_detect(&self, _game: &Game, _position: Position) {}
    fn on_player_left(&self, _game: &Game, _player: Player) {}
    fn on_receive_text(&self, _game: &Game, _player: Player, _text: &str) {}
    fn on_save_game(&self, _game: &Game, _game_name: &str) {}
    fn on_send_text(&self, _game: &Game, _text: &str) {}
    fn on_start(&self, _game: &Game) {}
    fn on_frame(&mut self, state: &Game);
    fn on_unit_create(&self, _game: &Game, _unit: Unit) {}
    fn on_unit_destroy(&self, _game: &Game, _unit: Unit) {}
    fn on_unit_discover(&self, _game: &Game, _unit: Unit) {}
    fn on_unit_complete(&self, _game: &Game, _unit: Unit) {}
    fn on_unit_evade(&self, _game: &Game, _unit: Unit) {}
    fn on_unit_hide(&self, _game: &Game, _unit: Unit) {}
    fn on_unit_morph(&self, _game: &Game, _unit: Unit) {}
    fn on_unit_renegade(&self, _game: &Game, _unit: Unit) {}
    fn on_unit_show(&self, _game: &Game, _unit: Unit) {}
}
