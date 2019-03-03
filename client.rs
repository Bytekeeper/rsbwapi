use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::thread;
use std::time::Duration;

use super::shm;
use crate::unit::Unit;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct Client<'a> {
    gameData: &'a mut BWAPI_GameData,
    pipe: File,
}

impl<'a> Client<'a> {
    pub fn new() -> Client<'a> {
        'outer: loop {
            if let Some(gameTable) = shm::mapMemory::<BWAPI_GameTable>("Local\\bwapi_shared_memory_game_list") {
                for game_instance in gameTable.gameInstances.iter() {
                    if game_instance.serverProcessID != 0 {
                        let pid = game_instance.serverProcessID;
                        let mut file: File = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .open(format!("\\\\.\\pipe\\bwapi_pipe_{}", pid))
                            .expect("Game table was found, but could not open bwapi_pipe!");
                        let mut buf: [u8; 1] = [0];
                        println!("Connecting to {}", pid);
                        loop {
                            file.read(&mut buf)
                                .expect("Could not read from bwapi_pipe!");
                            if buf[0] == 2 {
                                break;
                            }
                        }
                        println!("Connected to {}", pid);
                        let gameData = &format!("Local\\bwapi_shared_memory_{}", pid);
                        let gameData: &mut BWAPI_GameData = shm::mapMemory(gameData)
                            .expect("Game table was found, but could not establish shared memory link.");
                        println!("Client version: {}", gameData.client_version);
                        break 'outer Client {
                            gameData,
                            pipe: file,
                        };
                    }
                }
            } else {
                println!("Game table mapping not found.");
                thread::sleep(Duration::from_millis(1000));
            }
        }
    }

    pub fn isInGame(&self) -> bool {
        self.gameData.isInGame
    }

    pub fn getAllUnits(&self) -> Vec<Unit> {
        self.gameData.units[..self.gameData.initialUnitCount as usize].iter()
            .map(|x| Unit::new(x) )
            .collect()
    }

    pub fn update(&mut self) {
        let mut buf: [u8; 1] = [1];
        self.pipe.write(&buf)
            .expect("Pipe closed while writing");
        loop {
            self.pipe.read(&mut buf)
                .expect("Pipe closed while reading");
            if buf[0] == 2 {
                break;
            }
        }
    }
}