use crate::client::BWAPI_UnitData;
use crate::unittype::UnitType;
use num_traits::FromPrimitive;

#[derive(Debug)]
pub struct Unit<'a> {
    data: &'a BWAPI_UnitData
}

impl<'a> Unit<'a> {
    pub fn new(data: &'a BWAPI_UnitData) -> Unit<'a> {
        Unit {
            data
        }
    }

    pub fn getType(&self) -> UnitType {
        UnitType::from_i32(self.data.type_).unwrap_or(UnitType::None)
    }

    pub fn exists(&self) -> bool {
        self.data.exists
    }
}