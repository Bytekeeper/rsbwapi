use crate::game::Game;
use crate::player::Player;
use crate::projected::Projected;
use crate::unit::Unit;
use bwapi_wrapper::prelude::*;
use bwapi_wrapper::*;
use num_traits::FromPrimitive;

pub struct Bullet {
    id: usize,
    inner: Projected<Game, BWAPI_BulletData>,
}

impl Bullet {
    pub(crate) fn new(id: usize, game: Game, data: *const BWAPI_BulletData) -> Self {
        Self {
            id,
            inner: unsafe { Projected::new(game, data) },
        }
    }

    pub fn exists(&self) -> bool {
        self.inner.exists
    }

    pub fn get_angle(&self) -> f64 {
        self.inner.angle
    }

    pub fn get_player(&self) -> Option<Player> {
        self.inner.game().get_player(self.inner.player as usize)
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_position(&self) -> Option<Position> {
        Some(Position {
            x: self.inner.positionX,
            y: self.inner.positionY,
        })
    }

    pub fn get_remove_timer(&self) -> Option<i32> {
        if self.inner.removeTimer > 0 {
            Some(self.inner.removeTimer)
        } else {
            None
        }
    }

    pub fn get_source(&self) -> Option<Unit> {
        self.inner.game().get_unit(self.inner.source as usize)
    }

    pub fn get_target(&self) -> Option<Unit> {
        self.inner.game().get_unit(self.inner.target as usize)
    }

    pub fn get_target_position(&self) -> Option<Position> {
        Position::new_checked(
            self.inner.game(),
            self.inner.targetPositionX,
            self.inner.targetPositionY,
        )
    }

    pub fn get_type(&self) -> BulletType {
        BulletType::from_i32(self.inner.type_).unwrap()
    }

    pub fn get_velocity(&self) -> Option<Vector2D> {
        let result = Vector2D::new(self.inner.velocityX, self.inner.velocityY);
        if result.x == 0.0 && result.y == 0.0 {
            None
        } else {
            Some(result)
        }
    }

    pub fn is_visible(&self, player: &Player) -> bool {
        self.inner.isVisible[player.id]
    }
}

impl PartialEq for Bullet {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

type BulletType = BWAPI_BulletTypes_Enum_Enum;
