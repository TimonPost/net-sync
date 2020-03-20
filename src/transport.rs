mod client;
mod message;
mod packet;
mod postoffice;

use serde::{Serialize, Deserialize};
pub use self::{client::{Clients, Client, ClientId}, message::Message, packet::{SentPacket, ReceivedPacket, PostBoxMessage}, postoffice::{PostBox, PostOffice}};

/// Specification of urgency of the sending of a message. Typically we'll want to send messages
/// on simulation tick but the option to send messages immediately is available.
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub enum UrgencyRequirement {
    /// Message will be sent based on the current configuration of the simulation frame rate and
    /// the message send rate.
    OnTick,
    /// Message will be sent as soon as possible.
    Immediate,
}


//use crate::compression::{CompressionStrategy, ModificationCompressor};
//use std::net::UdpSocket;
//use track::{
//    serialization::{ModificationSerializer, SerialisationStrategy},
//    ModificationChannel,
//};
//
//pub struct Endpoint<S: SerialisationStrategy, C: CompressionStrategy> {
//    socket: UdpSocket,
//    channel: ModificationChannel,
//    compression: ModificationCompressor<C>,
//    serialization: ModificationSerializer<S>,
//}
//
////impl Endpoint {
////    pub fn recv_changes(&self) {
////
////    }
////
////    pub fn sent_changes(&self, addr: SocketAddr) {
////
////    }
////}
