use crate::types::UnitType;
use bwapi_wrapper::*;
use std::ffi::CStr;

#[derive(Clone, Copy)]
pub struct Player<'a> {
    pub id: usize,
    data: &'a BWAPI_PlayerData,
}

impl<'a> Player<'a> {
    pub(crate) fn new(id: usize, data: &'a BWAPI_PlayerData) -> Self {
        Player { id, data }
    }

    pub fn name(&self) -> &str {
        #[repr(C)]
        union Slices<'a> {
            u8: &'a [u8; 25],
            i8: &'a [i8; 25],
        }

        let name = unsafe {
            Slices {
                i8: &self.data.name,
            }
            .u8
        };
        CStr::from_bytes_with_nul(&name[..=name.iter().position(|&c| c == 0).unwrap()])
            .map(|n| n.to_str().unwrap())
            .unwrap()
    }

    pub fn armor(&self, _unit_type: UnitType) -> i32 {
        unimplemented!()
    }

    pub fn is_ally(&self, other: &Player) -> bool {
        self.data.isAlly[other.id]
    }
}

impl<'a> PartialEq for Player<'a> {
    fn eq(&self, other: &Player) -> bool {
        self.id == other.id
    }
}
