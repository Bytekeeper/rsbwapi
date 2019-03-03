#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#[macro_use]
extern crate num_derive;

mod shm;
pub mod client;
pub mod unittype;
pub mod unit;


fn main() {
    let mut client = client::Client::new();

    while !client.isInGame() {
        client.update();
    }

    for i in client.getAllUnits() {
        if i.exists() {
            println!("Unit {:?}", i);
            println!("Type {:?}", i.getType())
        }
    }
}
