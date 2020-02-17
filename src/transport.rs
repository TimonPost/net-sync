use crate::compression::{CompressionStrategy, ModificationCompressor};
use std::net::{SocketAddr, UdpSocket};
use track::{
    serialisation::{ModificationSerializer, SerialisationStrategy},
    ModificationChannel,
};

pub struct Endpoint<S: SerialisationStrategy, C: CompressionStrategy> {
    socket: UdpSocket,
    channel: ModificationChannel,
    compression: ModificationCompressor<C>,
    serialisation: ModificationSerializer<S>,
}

//impl Endpoint {
//    pub fn recv_changes(&self) {
//
//    }
//
//    pub fn sent_changes(&self, addr: SocketAddr) {
//
//    }
//}
