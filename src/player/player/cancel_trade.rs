use eolib::{data::EoWriter, protocol::net::{PacketAction, PacketFamily}};

use super::Player;

impl Player {
    pub async fn cancel_trade(&mut self) {
        let interact_player_id = match self.interact_player_id {
            Some(player_id) => player_id,
            None => return,
        };

        self.interact_player_id = None;
        self.trading = false;
        self.trade_accepted = false;

        let mut writer = EoWriter::new();
        writer.add_short(interact_player_id);
        let _ = self
            .bus
            .send(PacketAction::Close, PacketFamily::Trade, writer.to_byte_array())
            .await;
    }
}
