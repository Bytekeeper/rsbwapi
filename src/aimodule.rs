use crate::game::{Commands, Game};

pub trait AiModule {
    fn onStart(&self, game: &mut Game);
    fn onFrame(&mut self, state: &Game, cmd: &mut Commands);
}
