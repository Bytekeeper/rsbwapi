use crate::game::{Commands, Game};
use crate::unit::Unit;

pub trait AiModule {
    fn on_start(&self, game: &Game);
    fn on_frame(&mut self, state: &Game, cmd: &mut Commands);
    fn on_unit_create(&self, _game: &Game, _cmd: &mut Commands, _unit: &Unit) {}
    fn on_unit_destroy(&self, _game: &Game, _cmd: &mut Commands, _unit: &Unit) {}
    fn on_unit_discover(&self, _game: &Game, _cmd: &mut Commands, _unit: &Unit) {}
    fn on_unit_complete(&self, _game: &Game, _cmd: &mut Commands, _unit: &Unit) {}
    fn on_unit_evade(&self, _game: &Game, _cmd: &mut Commands, _unit: &Unit) {}
    fn on_unit_hide(&self, _game: &Game, _cmd: &mut Commands, _unit: &Unit) {}
    fn on_unit_morph(&self, _game: &Game, _cmd: &mut Commands, _unit: &Unit) {}
    fn on_unit_renegade(&self, _game: &Game, _cmd: &mut Commands, _unit: &Unit) {}
    fn on_unit_show(&self, _game: &Game, _cmd: &mut Commands, _unit: &Unit) {}
}
