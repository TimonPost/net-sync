use crate::ClientMessage;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use serde::export::fmt::Debug;

pub trait PostBoxMessage : Serialize + for<'a> Deserialize<'a> + Debug {

}

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct SentPacket {
    /// The event that defines what kind of packet this is.
    event: ClientMessage,
}

impl SentPacket {
    pub fn new(event: ClientMessage) -> SentPacket {
        SentPacket { event }
    }

    pub fn event(&self) -> &ClientMessage {
        &self.event
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceivedPacket {
    addr: SocketAddr,
    event: ClientMessage,
}

impl ReceivedPacket {
    pub fn new(addr: SocketAddr, packet: SentPacket) -> Self {
        ReceivedPacket {
            event: packet.event,
            addr,
        }
    }

    pub fn source(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn event(&self) -> ClientMessage {
        self.event.clone()
    }
}

#[cfg(test)]
pub mod test {
    use crate::{ClientMessage, SentPacket};
    use net_sync::uid::Uid;

    #[test]
    fn create_sent_packet_test() {
        let id = Uid(0);
        let event = ClientMessage::EntityRemoved(id);

        let packet = SentPacket::new(event.clone());
        assert_eq!(packet.event(), &event);
    }
}
