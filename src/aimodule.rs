use crate::game::{Commands, Game};

pub trait AiModule {
    fn on_start(&self, game: &mut Game);
    fn on_frame(&mut self, state: &Game, cmd: &mut Commands);
}
