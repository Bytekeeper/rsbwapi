#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;

mod shm;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() -> std::io::Result<()> {
    let gameTable: &mut BWAPI_GameTable = shm::mapMemory("Local\\bwapi_shared_memory_game_list");

    for game_instance in gameTable.gameInstances.iter() {
        if game_instance.serverProcessID != 0 {
            let pid = game_instance.serverProcessID;
            let mut file: File = OpenOptions::new()
                .read(true)
                .write(true)
                .open(format!("\\\\.\\pipe\\bwapi_pipe_{}", pid))?;
            let mut buf: [u8; 1] = [0];
            println!("Connecting to {}", pid);
            loop {
                file.read(&mut buf)?;
                if buf[0] == 2 {
                    break;
                }
            }
            println!("Connected to {}", pid);
            let gameData = &format!("Local\\bwapi_shared_memory_{}", pid);
            let gameData: &mut BWAPI_GameData = shm::mapMemory(gameData);
            println!("Client version: {}", gameData.client_version);
        }
    }

    Ok(())
}
