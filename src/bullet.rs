use crate::player::Player;
use crate::types::*;
use crate::unit::Unit;
use crate::*;
use bwapi_wrapper::*;
use num_traits::FromPrimitive;

pub struct Bullet<'a> {
    id: usize,
    game: &'a Game,
    data: &'a BWAPI_BulletData,
}

impl<'a> Bullet<'a> {
    pub fn new(id: usize, game: &'a Game, data: &'a BWAPI_BulletData) -> Self {
        Self { id, game, data }
    }

    pub fn exists(&self) -> bool {
        self.data.exists
    }

    pub fn get_player(&self) -> Option<Player> {
        self.game.get_player(self.data.player)
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_position(&self) -> Option<Position> {
        Some(Position {
            x: self.data.positionX,
            y: self.data.positionY,
        })
    }

    pub fn get_remove_timer(&self) -> Option<i32> {
        if self.data.removeTimer > 0 {
            Some(self.data.removeTimer)
        } else {
            None
        }
    }

    pub fn get_source(&self) -> Option<Unit> {
        self.game.get_unit(self.data.source)
    }

    pub fn get_target(&self) -> Option<Unit> {
        self.game.get_unit(self.data.target)
    }

    pub fn get_target_position(&self) -> Option<Position> {
        Some(Position {
            x: self.data.targetPositionX,
            y: self.data.targetPositionY,
        })
    }

    pub fn get_type(&self) -> BulletType {
        BulletType::from_i32(self.data.type_).unwrap()
    }

    pub fn get_velocity(&self) -> Vector2D {
        Vector2D {
            x: self.data.velocityX,
            y: self.data.velocityY,
        }
    }

    pub fn is_visible(&self, player: &Player) -> bool {
        self.data.isVisible[player.id]
    }
}

type BulletType = BWAPI_BulletTypes_Enum_Enum;
