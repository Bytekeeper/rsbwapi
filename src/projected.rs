use crate::Game;
use std::ops::Deref;

#[derive(Clone)]
pub(crate) struct Projected<G, T> {
    owner: G,
    data: *const T,
}

impl<G, T> Deref for Projected<G, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Unsafe! The reference is inside shared memory "held" by Game.
        // And RSBWAPI will only mutate in one function (as will BWAPI externally).
        unsafe { &*self.data }
    }
}

impl<G, T> Projected<G, T> {
    pub(crate) unsafe fn new(owner: G, data: *const T) -> Self {
        Self { owner, data }
    }

    pub(crate) fn owner(&self) -> &G {
        &self.owner
    }
}

impl<T> Projected<Game, T> {
    pub(crate) fn game(&self) -> &Game {
        self.owner()
    }
}
