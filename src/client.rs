use crate::game::Game;
use bwapi_wrapper::*;

use std::ffi::CStr;
#[cfg(windows)]
use std::fs::File;
#[cfg(windows)]
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

use std::thread;
use std::time::Duration;

use crate::aimodule::AiModule;

use super::shm;

pub struct Client {
    pipe: SyncStream,
    game: Game,
}

#[cfg(windows)]
type SyncStream = File;
#[cfg(unix)]
type SyncStream = UnixStream;

pub trait ToStr {
    fn to_str(&self) -> &str;
}

impl ToStr for [i8] {
    fn to_str(&self) -> &str {
        let as_u8 = unsafe { &*(self as *const [i8] as *const [u8]) };
        let nul_pos = memchr::memchr(0, as_u8);
        if let Some(nul_pos) = nul_pos {
            unsafe { CStr::from_bytes_with_nul_unchecked(&as_u8[0..nul_pos + 1]) }
                .to_str()
                .unwrap()
        } else {
            ""
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        #[cfg(unix)]
        let selected_pid = std::env::var("RSBWAPI_SERVER_PID")
            .ok()
            .map(|pid| pid.parse::<u32>().expect("Invalid RSBWAPI_SERVER_PID"));
        'outer: loop {
            #[cfg(windows)]
            if let Some(game_table) =
                shm::map_memory::<BWAPI_GameTable>("Local\\bwapi_shared_memory_game_list")
            {
                let game_table = game_table.get();
                for game_instance in game_table.gameInstances.iter() {
                    if game_instance.serverProcessID != 0 && !game_instance.isConnected {
                        let pid = game_instance.serverProcessID;
                        let mut file: File = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .open(format!("\\\\.\\pipe\\bwapi_pipe_{}", pid))
                            .expect("Game table was found, but could not open bwapi_pipe!");
                        let mut buf: [u8; 1] = [0];
                        println!("Connecting to {}", pid);
                        loop {
                            file.read_exact(&mut buf)
                                .expect("Could not read from bwapi_pipe!");
                            if buf[0] == 2 {
                                break;
                            }
                        }
                        println!("Connected to {}", pid);
                        let game_data = &format!("Local\\bwapi_shared_memory_{}", pid);
                        let game_data: shm::Shm<BWAPI_GameData> = shm::map_memory(game_data)
                            .expect(
                                "Game table was found, but could not establish shared memory link.",
                            );
                        println!("Client version: {}", game_data.get().client_version);
                        break 'outer Client {
                            pipe: file,
                            game: Game::new(game_data),
                        };
                    }
                }
            } else {
                println!("Game table mapping not found.");
                thread::sleep(Duration::from_millis(1000));
            }

            #[cfg(unix)]
            if let Some(pid) = selected_pid {
                if let Some(game_data) =
                    shm::map_memory::<BWAPI_GameData>(&format!("/bwapi_shared_memory_{pid}"))
                {
                    if let Ok(mut pipe) = UnixStream::connect(format!("/tmp/bwapi_socket_{pid}")) {
                        let mut code = [0];
                        if pipe.read_exact(&mut code).is_ok() && code[0] == 2 {
                            println!("Connected to {pid}");
                            break 'outer Client {
                                pipe,
                                game: Game::new(game_data),
                            };
                        }
                    }
                }
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            #[cfg(unix)]
            if let Some(game_table) =
                shm::map_memory::<BWAPI_GameTable>("/bwapi_shared_memory_game_list")
            {
                let mut instances: Vec<_> = game_table
                    .get()
                    .gameInstances
                    .iter()
                    .filter(|instance| {
                        instance.serverProcessID != 0
                            && !instance.isConnected
                            && selected_pid.is_none_or(|pid| instance.serverProcessID == pid)
                    })
                    .collect();
                instances.sort_by_key(|instance| std::cmp::Reverse(instance.lastKeepAliveTime));
                for instance in instances {
                    let pid = instance.serverProcessID;
                    // OpenBW unlinks the game mapping as soon as it accepts the socket.
                    let Some(game_data) =
                        shm::map_memory::<BWAPI_GameData>(&format!("/bwapi_shared_memory_{pid}"))
                    else {
                        continue;
                    };
                    let Ok(mut pipe) = UnixStream::connect(format!("/tmp/bwapi_socket_{pid}"))
                    else {
                        continue;
                    };
                    let mut code = [0];
                    if pipe.read_exact(&mut code).is_err() || code[0] != 2 {
                        continue;
                    }
                    println!("Connected to {pid}");
                    break 'outer Client {
                        pipe,
                        game: Game::new(game_data),
                    };
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}
impl Client {
    pub fn update(&mut self, module: &mut impl AiModule) {
        let mut buf: [u8; 1] = [1];
        self.pipe
            .write_all(&buf)
            .expect("Pipe closed while writing");
        loop {
            self.pipe
                .read_exact(&mut buf)
                .expect("Pipe closed while reading");
            if buf[0] == 2 {
                break;
            }
        }
        self.game.handle_events(module);
    }

    pub fn get_game(&self) -> &Game {
        &self.game
    }
}
