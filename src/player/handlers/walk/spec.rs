use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::walk::Spec,
};

use crate::{player::PlayerHandle, Bytes};

pub async fn spec(buf: Bytes, player: PlayerHandle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut packet = Spec::default();
    let reader = StreamReader::new(buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    // TODO: handle anti-speed

    if let Ok(map) = player.get_map().await {
        let player_id = player.get_player_id().await?;
        map.walk(
            player_id,
            packet.walk.direction,
        );
    }

    Ok(())
}
