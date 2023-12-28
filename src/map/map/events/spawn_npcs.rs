use std::cmp;

use chrono::Duration;
use eolib::protocol::{Coords, Direction};
use rand::Rng;

use crate::{map::NPCBuilder, NPC_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub async fn spawn_npcs(&mut self) {
        if self.file.npcs.is_empty() {
            return;
        }

        let now = chrono::Utc::now();
        if self.npcs.is_empty() {
            let mut npc_index: i32 = 0;

            let dead_since = if SETTINGS.npcs.instant_spawn {
                now - Duration::days(1)
            } else {
                now
            };

            for (spawn_index, spawn) in self.file.npcs.iter().enumerate() {
                let data_record = match NPC_DB.npcs.get(spawn.id as usize - 1) {
                    Some(npc) => npc,
                    None => {
                        error!(
                            "Failed to load NPC {} (Map: {}, Coords: {:?})",
                            spawn.id, self.id, spawn.coords,
                        );
                        continue;
                    }
                };

                for _ in 0..spawn.amount as i64 {
                    self.npcs.insert(
                        npc_index,
                        NPCBuilder::new()
                            .id(spawn.id)
                            .coords(Coords::default())
                            .direction(Direction::Down)
                            .spawn_index(spawn_index)
                            .alive(false)
                            .dead_since(dead_since)
                            .hp(data_record.hp)
                            .max_hp(data_record.hp)
                            .build(),
                    );
                    npc_index += 1;
                }
            }
        }

        let mut rng = rand::thread_rng();
        let indexes = self.npcs.keys().cloned().collect::<Vec<i32>>();
        for index in indexes {
            let (alive, spawn_time, dead_since, spawn_coords, spawn_type) = {
                match self.npcs.get(&index) {
                    Some(npc) => {
                        let spawn = &self.file.npcs[npc.spawn_index];
                        (
                            npc.alive,
                            spawn.spawn_time,
                            npc.dead_since,
                            spawn.coords,
                            spawn.spawn_type,
                        )
                    }
                    None => continue,
                }
            };

            if alive || now.timestamp() - dead_since.timestamp() < spawn_time.into() {
                continue;
            }

            let file_spawn_coords = spawn_coords;
            let mut spawn_coords = if spawn_type == 7 {
                spawn_coords
            } else {
                Coords {
                    x: cmp::max(
                        cmp::min(
                            spawn_coords.x + rng.gen_range(-2..=2),
                            self.file.width,
                        ),
                        0,
                    ) as i32,
                    y: cmp::max(
                        cmp::min(
                            spawn_coords.y + rng.gen_range(-2..=2),
                            self.file.height,
                        ),
                        0,
                    ) as i32,
                }
            };

            let mut i = 0;
            while !self.is_tile_walkable_npc(&spawn_coords)
                && (i > 100 || !self.is_tile_occupied(&spawn_coords))
            {
                let x = cmp::max(
                    cmp::min(
                        file_spawn_coords.x + rng.gen_range(-2..=2),
                        self.file.width,
                    ),
                    0,
                );
                let y = cmp::max(
                    cmp::min(
                        file_spawn_coords.y + rng.gen_range(-2..=2),
                        self.file.height,
                    ),
                    0,
                );
                spawn_coords = Coords {
                    x: x as i32,
                    y: y as i32,
                };

                i += 1;

                if i >= 200 {
                    break;
                }
            }

            let npc = match self.npcs.get_mut(&index) {
                Some(npc) => npc,
                None => continue,
            };

            npc.alive = true;
            npc.hp = npc.max_hp;
            npc.coords = spawn_coords;
            npc.direction = if spawn_type == 7 {
                Direction::from(spawn_type & 0x03)
            } else {
                match rand::random::<u8>() % 4 {
                    0 => Direction::Down,
                    1 => Direction::Left,
                    2 => Direction::Up,
                    3 => Direction::Right,
                    _ => unreachable!(),
                }
            }
        }
    }
}
