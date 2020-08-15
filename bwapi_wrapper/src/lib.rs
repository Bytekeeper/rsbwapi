#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use num_derive::FromPrimitive;

pub mod command;
pub mod tech_type;
pub mod unit_type;
pub mod upgrade_type;
pub mod weapon_type;

pub mod position;
pub mod prelude;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub trait TypeFrom {
    fn new(i: i32) -> Self;
}

impl<T: num_traits::FromPrimitive> TypeFrom for T {
    fn new(i: i32) -> Self {
        Self::from_i32(i).unwrap()
    }
}
