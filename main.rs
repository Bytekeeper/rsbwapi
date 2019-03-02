#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use rand::Rng;
use std::io;
use std::cmp::Ordering;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    println!("Guess the number!");

    let secret = rand::thread_rng().gen_range(1, 101);
    let test = BWAPI_GameData::default();

    loop {
        println!("Please !! Number!! :");

        let mut guess = String::new();
        io::stdin().read_line(&mut guess)
            .expect("Wtf?");
        let guess : u32 = match guess.trim().parse() {
            Ok(x) => x,
            Err(_) => {
                println!("Try agaain");
                continue
            }
        };

        println!("You guessed {}", guess);

        match guess.cmp(&secret) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("Hit!");
                break;
            }
        }
    }
}
