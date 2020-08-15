use crate::prelude::*;
use crate::TypeFrom;

impl UnitCommand {
    pub fn get_type(&self) -> UnitCommandType {
        UnitCommandType::new(self.type_._base as i32)
    }

    pub fn get_unit_type(&self) -> UnitType {
        match self.get_type() {
            UnitCommandType::Build
            | UnitCommandType::Build_Addon
            | UnitCommandType::Train
            | UnitCommandType::Morph => UnitType::new(self.extra),
            _ => UnitType::None,
        }
    }

    pub fn get_target_position(&self) -> Position {
        match self.get_type() {
            UnitCommandType::Build | UnitCommandType::Land | UnitCommandType::Place_COP => {
                TilePosition {
                    x: self.x,
                    y: self.y,
                }
                .to_position()
            }
            _ => Position {
                x: self.x,
                y: self.y,
            },
        }
    }

    pub fn get_target_tile_position(&self) -> TilePosition {
        match self.get_type() {
            UnitCommandType::Build | UnitCommandType::Land | UnitCommandType::Place_COP => {
                TilePosition {
                    x: self.x,
                    y: self.y,
                }
            }
            _ => Position {
                x: self.x,
                y: self.y,
            }
            .to_tile_position(),
        }
    }

    pub fn get_tech_type(&self) -> TechType {
        match self.get_type() {
            UnitCommandType::Research
            | UnitCommandType::Use_Tech
            | UnitCommandType::Use_Tech_Position
            | UnitCommandType::Use_Tech_Unit => TechType::new(self.extra),
            _ => TechType::None,
        }
    }

    pub fn get_slot(&self) -> Option<i32> {
        if self.get_type() == UnitCommandType::Cancel_Train_Slot {
            Some(self.extra)
        } else {
            None
        }
    }

    pub fn is_queued(&self) -> bool {
        match self.get_type() {
            UnitCommandType::Attack_Move
            | UnitCommandType::Attack_Unit
            | UnitCommandType::Move
            | UnitCommandType::Patrol
            | UnitCommandType::Hold_Position
            | UnitCommandType::Stop
            | UnitCommandType::Follow
            | UnitCommandType::Gather
            | UnitCommandType::Return_Cargo
            | UnitCommandType::Repair
            | UnitCommandType::Load
            | UnitCommandType::Unload_All
            | UnitCommandType::Unload_All_Position
            | UnitCommandType::Right_Click_Position
            | UnitCommandType::Right_Click_Unit => self.extra != 0,
            _ => false,
        }
    }
}
