use std::{collections::HashMap, sync::Arc};

use eo::{
    data::{map::MapFile, EOShort, Serializeable},
    net::{
        packets::server::{avatar, face, map_info, players},
        Action, Family, NearbyInfo,
    },
};
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};

use crate::player::PlayerHandle;

use super::{Command, Item, NPC};

pub struct Map {
    pub rx: UnboundedReceiver<Command>,
    file: MapFile,
    items: Arc<Mutex<Vec<Item>>>,
    npcs: Arc<Mutex<Vec<NPC>>>,
    players: Arc<Mutex<HashMap<EOShort, PlayerHandle>>>,
}

impl Map {
    pub fn new(file: MapFile, rx: UnboundedReceiver<Command>) -> Self {
        Self {
            file,
            rx,
            items: Arc::new(Mutex::new(Vec::new())),
            npcs: Arc::new(Mutex::new(Vec::new())),
            players: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::DropPlayer(target_player_id, coords) => {
                let mut players = self.players.lock().await;
                players.remove(&target_player_id).unwrap();
                let packet = avatar::Remove {
                    player_id: target_player_id,
                    warp_animation: None,
                };
                let buf = packet.serialize();
                for player in players.values() {
                    if player.is_in_range(coords).await {
                        player.send(Action::Remove, Family::Avatar, buf.clone());
                    }
                }
            }
            Command::Enter(target_player_id, target) => {
                let character_map_info = target.get_character_map_info().await.unwrap();
                let packet = players::Agree::new(character_map_info);
                let buf = packet.serialize();
                let mut players = self.players.lock().await;
                for player in players.values() {
                    if target.is_in_range(player.get_coords().await.unwrap()).await {
                        player.send(Action::Agree, Family::Players, buf.clone());
                    }
                }
                players.insert(target_player_id, target);
            }
            Command::Face(target_player_id, direction) => {
                let packet = face::Player::new(target_player_id, direction);
                let buf = packet.serialize();
                let players = self.players.lock().await;
                let target = players.get(&target_player_id).unwrap();
                for player in players.values() {
                    let player_id = player.get_player_id().await;
                    // TODO: don't unwrap
                    if target_player_id != player_id
                        && target.is_in_range(player.get_coords().await.unwrap()).await
                    {
                        player.send(Action::Player, Family::Face, buf.clone());
                    }
                }
            }
            Command::GetCharacterMapInfo {
                player_id,
                respond_to,
            } => {
                let players = self.players.lock().await;
                let player = players.get(&player_id).unwrap();
                let character_info = match player.get_character_map_info().await {
                    Ok(character_info) => character_info,
                    Err(e) => {
                        warn!(
                            "Requested character map info for player {} failed: {}",
                            player_id, e
                        );
                        let _ = respond_to.send(Err(Box::new(e)));
                        return;
                    }
                };

                let reply = map_info::Reply::character(character_info);
                let _ = respond_to.send(Ok(reply));
            }
            Command::GetHashAndSize { respond_to } => {
                let _ = respond_to.send((self.file.hash, self.file.size));
            }
            Command::GetNearbyInfo {
                target_player_id,
                respond_to,
            } => {
                let players = self.players.lock().await;
                let target = players.get(&target_player_id).unwrap();
                let items = self.items.lock().await;
                let npcs = self.npcs.lock().await;
                let mut nearby_items = Vec::new();
                let mut nearby_npcs = Vec::new();
                let mut nearby_characters = Vec::new();
                for item in items.iter() {
                    if target.is_in_range(item.coords).await {
                        nearby_items.push(item.to_item_map_info());
                    }
                }
                for npc in npcs.iter() {
                    if target.is_in_range(npc.coords.to_coords()).await {
                        nearby_npcs.push(npc.to_npc_map_info());
                    }
                }
                for player in players.values() {
                    let player_id = player.get_player_id().await;
                    // TODO: don't unwrap
                    if target_player_id == player_id
                        || target.is_in_range(player.get_coords().await.unwrap()).await
                    {
                        nearby_characters.push(player.get_character_map_info().await.unwrap());
                    }
                }
                let _ = respond_to.send(NearbyInfo {
                    items: nearby_items,
                    npcs: nearby_npcs,
                    characters: nearby_characters,
                });
            }
            Command::Leave {
                target_player_id,
                warp_animation,
                respond_to,
            } => {
                let mut players = self.players.lock().await;
                let target = players.remove(&target_player_id).unwrap();
                let packet = avatar::Remove {
                    player_id: target_player_id,
                    warp_animation,
                };
                let buf = packet.serialize();
                for player in players.values() {
                    if target.is_in_range(player.get_coords().await.unwrap()).await {
                        player.send(Action::Remove, Family::Avatar, buf.clone());
                    }
                }
                let _ = respond_to.send(());
            }
            Command::Serialize { respond_to } => {
                let _ = respond_to.send(self.file.serialize());
            }
        }
    }
}
