use eo::{
    data::{Serializeable, StreamReader},
    net::{packets::client::map_info::Request, Action, Family},
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn request(buf: PacketBuf, player: PlayerHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    if let Ok(map) = player.get_map().await {
        let reply = map
            .get_map_info(request.player_ids, request.npc_indexes)
            .await;
        if reply.characters.is_some() || reply.npcs.is_some() {
            debug!("Reply: {:?}", reply);
            player.send(Action::Reply, Family::MapInfo, reply.serialize());
        }
    }
}