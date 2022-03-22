use crate::game::Game;
use crate::player::Player;
use crate::unit::Unit;
use bwapi_wrapper::prelude::*;
use bwapi_wrapper::*;
use num_traits::FromPrimitive;
use std::cell::Ref;

pub struct Bullet {
    id: usize,
    game: Game,
}

impl Bullet {
    pub fn new(id: usize, game: Game) -> Self {
        Self { id, game }
    }

    fn data(&self) -> Ref<'_, BWAPI_BulletData> {
        Ref::map(self.game.inner.borrow(), |d| &d.data.bullets[self.id])
    }

    pub fn exists(&self) -> bool {
        self.data().exists
    }

    pub fn get_angle(&self) -> f64 {
        self.data().angle
    }

    pub fn get_player(&self) -> Option<Player> {
        self.game.get_player(self.data().player as usize)
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_position(&self) -> Option<Position> {
        Some(Position {
            x: self.data().positionX,
            y: self.data().positionY,
        })
    }

    pub fn get_remove_timer(&self) -> Option<i32> {
        if self.data().removeTimer > 0 {
            Some(self.data().removeTimer)
        } else {
            None
        }
    }

    pub fn get_source(&self) -> Option<Unit> {
        self.game.get_unit(self.data().source as usize)
    }

    pub fn get_target(&self) -> Option<Unit> {
        self.game.get_unit(self.data().target as usize)
    }

    pub fn get_target_position(&self) -> Option<Position> {
        Position::new_checked(
            &self.game,
            self.data().targetPositionX,
            self.data().targetPositionY,
        )
    }

    pub fn get_type(&self) -> BulletType {
        BulletType::from_i32(self.data().type_).unwrap()
    }

    pub fn get_velocity(&self) -> Option<Vector2D> {
        let result = Vector2D::new(self.data().velocityX, self.data().velocityY);
        if result.x == 0.0 && result.y == 0.0 {
            None
        } else {
            Some(result)
        }
    }

    pub fn is_visible(&self, player: &Player) -> bool {
        self.data().isVisible[player.id]
    }
}

impl PartialEq for Bullet {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

type BulletType = BWAPI_BulletTypes_Enum_Enum;
