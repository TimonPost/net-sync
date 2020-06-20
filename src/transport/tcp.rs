use std::{collections::{hash_map::IterMut, HashMap}, io::Write, net::{SocketAddr, TcpListener, TcpStream}, io};

use std::collections::hash_map::{Iter, Keys};
use crate::serialization::SerializationStrategy;
use crate::compression::CompressionStrategy;
use crate::synchronisation::{NetworkMessage, NetworkCommand, CommandFrame};
use crate::packer::Packer;
use crate::transport::{PostOffice, PostBox};
use crate::transport;
use crate::event::{NetworkEventQueue, NetworkEvent};
use std::io::{Read, Error};
use log::{error, debug};
use crate::error::{ErrorKind};

pub struct TcpClientResource {
    stream: TcpStream,
    connected: bool,
}

impl TcpClientResource {
    pub fn new(addr: SocketAddr) ->  Result<TcpClientResource, ErrorKind> {
        let stream = TcpStream::connect(addr).unwrap();
        stream.set_nonblocking(true).unwrap();

        Ok(TcpClientResource { stream, connected: true })
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }

    pub fn sent(&mut self, data: &[u8]) -> Result<usize, ErrorKind> {
        Ok(self.stream.write(data)?)
    }

    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }

    pub fn stream_mut(&mut self) -> &TcpStream {
        &mut self.stream
    }

    pub fn addr(&self) -> Result<SocketAddr, ErrorKind> {
        Ok(self.stream.local_addr()?)
    }
}

pub struct TcpListenerResource {
    listener: Option<TcpListener>,
    streams: HashMap<SocketAddr, (bool, TcpStream)>,
}

impl TcpListenerResource {
    pub fn new(listener: Option<TcpListener>) -> Self {
        Self {
            listener,
            streams: HashMap::new(),
        }
    }

    /// Returns an immutable reference to the listener if there is one configured.
    pub fn get(&self) -> Option<&TcpListener> {
        self.listener.as_ref()
    }

    /// Returns a mutable reference to the listener if there is one configured.
    pub fn get_mut(&mut self) -> Option<&mut TcpListener> {
        self.listener.as_mut()
    }

    /// Sets the bound listener to the `TcpNetworkResource`.
    pub fn set_listener(&mut self, listener: TcpListener) {
        self.listener = Some(listener);
    }

    /// Drops the listener from the `TcpNetworkResource`.
    pub fn drop_listener(&mut self) {
        self.listener = None;
    }

    /// Returns a tuple of an active TcpStream and whether ot not that stream is active
    pub fn get_stream(&mut self, addr: SocketAddr) -> Option<&mut (bool, TcpStream)> {
        self.streams.get_mut(&addr)
    }

    /// Registers an new incoming stream to the TCP listener.
    pub fn register_stream(&mut self, addr: SocketAddr, stream: TcpStream) {
        self.streams.insert(addr, (true, stream));
    }

    /// Drops the stream with the given `SocketAddr`. This will be called when a peer seems to have
    /// been disconnected
    pub fn drop_stream(&mut self, addr: SocketAddr) -> Option<(bool, TcpStream)> {
        self.streams.remove(&addr)
    }

    /// Returns an iterator over the Tcp listener its streams.
    pub fn iter(&self) -> Iter<'_, SocketAddr, (bool, TcpStream)> {
        self.streams.iter()
    }

    /// Returns a mutable iterator over the Tcp listener its streams.
    pub fn iter_mut(&mut self) -> IterMut<'_, SocketAddr, (bool, TcpStream)> {
        self.streams.iter_mut()
    }

    pub fn addresses(&self) -> Keys<SocketAddr, (bool, TcpStream)> {
        self.streams.keys()
    }
}

pub fn tcp_connection_listener<
    ServerToClientMessage: NetworkMessage,
    ClientToServerMessage: NetworkMessage,
    ClientToServerCommand: NetworkCommand,
>(tcp: &mut TcpListenerResource, postoffice: &mut PostOffice<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>, network_events: &mut NetworkEventQueue) {
    if !tcp.get().is_some() {
        return;
    }

    loop {
        let (stream, addr) = match tcp.get().unwrap().accept() {
            Ok((stream, addr)) => {
                stream
                    .set_nonblocking(true)
                    .expect("Setting nonblocking mode");
                stream.set_nodelay(true).expect("Setting nodelay");

                debug!("Incoming TCP connection: {:?}", addr);

                (stream, addr)
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => {
                error!("Error while handling TCP connection: {:?}", e);
                // TODO: handle error
                break;
            }
        };

        tcp.register_stream(addr, stream);
        postoffice.add_client(addr);
        network_events.enqueue(NetworkEvent::Connected(addr))
    }
}


pub fn tcp_client_receive_system<
    S: SerializationStrategy + 'static,
    C: CompressionStrategy + 'static,
    ServerToClientMessage: NetworkMessage,
    ClientToServerMessage: NetworkMessage,
    ClientToServerCommand: NetworkCommand,
>(tcp: &mut TcpClientResource, postbox: &mut PostBox<transport::ServerToClientMessage<ServerToClientMessage>,transport::ClientToServerMessage<ClientToServerMessage, ClientToServerCommand>>, packer: &Packer<S, C>, network_events: &mut NetworkEventQueue, recv_buffer: &mut Vec<u8>) {
    let result = tcp.stream().read(recv_buffer);

    match result {
        Ok(recv_len) => {
            if recv_len < 5 {
                return;
            }

            // match unpacker
            //     .compression()
            //     .decompress(&recv_buffer[..recv_len]) {
            //     Ok(decompressed) => {
            match packer.serialization().deserialize::<Vec<
                transport::ServerToClientMessage<ServerToClientMessage>,
            >>(&recv_buffer[..recv_len])
            {
                Ok(deserialized) => {
                    debug!("Received {} bytes from server.", recv_len);
                    for packet in deserialized.into_iter() {
                        postbox.add_to_inbox(packet);
                    }
                }
                Err(e) => {
                    error!(
                        "Error occurred when deserializing TCP-packet. Reason: {:?}",
                        e
                    );
                }
            }
            //     }
            //     Err(e) => {
            //         error!("Error occurred when decompressing TCP-packet. Reason: {:?}", e);
            //     }
            // }
        }
        Err(e) => {
            match e.kind() {
                io::ErrorKind::ConnectionReset => {
                    let addr = tcp.addr().expect("Can not read client local socket address.");
                    tcp.set_connected(false);
                    network_events.enqueue(NetworkEvent::Disconnected(addr, 0)) // TODO: replace with current client id
                }
                io::ErrorKind::WouldBlock => {}
                _ => {
                    error!("Error occurred when receiving TCP-packet {}", e)
                }
            };
        }
    }
}

pub fn tcp_client_sent_system<
    S: SerializationStrategy + 'static,
    C: CompressionStrategy + 'static,
    ServerToClientMessage: NetworkMessage,
    ClientToServerMessage: NetworkMessage,
    ClientToServerCommand: NetworkCommand,
>(tcp: &mut TcpClientResource, postbox: &mut PostBox<transport::ServerToClientMessage<ServerToClientMessage>,transport::ClientToServerMessage<ClientToServerMessage, ClientToServerCommand>>,  packer: &Packer<S, C>,  network_events: &mut NetworkEventQueue) {
    if postbox.empty_outgoing() {
        return;
    }
    let packets =
        postbox.drain_outgoing(|_| true).into_iter().collect::<Vec<
            transport::ClientToServerMessage<ClientToServerMessage, ClientToServerCommand>,
        >>();

    if packets.len() == 0 {
        return;
    }

    match &packer.serialization().serialize(&packets) {
        Ok(serialized) => {
            debug!("Sending {} packets to host.", packets.len());
            // let compressed = packer.compression().compress(&serialized);
            //
            // if let Err(e) = tcp_client.sent(&compressed) {
            //     error!("Error occurred when sending TCP-packet. Reason: {:?}", e);
            // }

            if let Err(e) = tcp.sent(&serialized) {
                match e {
                    ErrorKind::IoError(e) => {
                        match e.kind() {
                            io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe => {
                                let addr = tcp.addr().expect("Failed tor read client local socket addr.");
                                tcp.set_connected(false);
                                network_events.enqueue(NetworkEvent::Disconnected(addr, 0)) // TODO: replace with current client id
                            },
                            _ => {
                                error!("Error occurred when sending TCP-packet. Reason: {:?}", e);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            error!(
                "Error occurred when serializing TCP-packet. Reason: {:?}",
                e
            );
        }
    }
}

pub fn tcp_server_receive_system<
    S: SerializationStrategy + 'static,
    C: CompressionStrategy + 'static,
    ServerToClientMessage: NetworkMessage,
    ClientToServerMessage: NetworkMessage,
    ClientToServerCommand: NetworkCommand
>(tcp: &mut TcpListenerResource, postoffice: &mut PostOffice<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>, packer: &Packer<S, C>, command_frame: CommandFrame, network_events: &mut NetworkEventQueue, recv_buffer: &mut Vec<u8>) {
    for (_, (active, stream)) in tcp.iter_mut() {
        // If we can't get a peer_addr, there is likely something pretty wrong with the
        // connection so we'll mark it inactive.
        let peer_addr = match stream.peer_addr() {
            Ok(addr) => addr,
            Err(_e) => {
                *active = false;
                continue;
            }
        };

        loop {
            let result = stream.read(recv_buffer);

            let client = postoffice
                .client_by_addr_mut(&peer_addr)
                .expect("Client should exist");

            match result {
                Ok(recv_len) => {
                    if recv_len < 5 {
                        *active = false;
                        break;
                    }

                    debug!(
                        "Received {} bytes from TCP stream: {:?}.",
                        recv_len, peer_addr
                    );

                    let buffer = &recv_buffer[..recv_len];

                    // match unpacker
                    //     .compression()
                    //     .decompress(buffer) {
                    //     Ok(decompressed) => {
                    match packer
                        .serialization()
                        .deserialize::<Vec<transport::ClientToServerMessage<ClientToServerMessage, ClientToServerCommand>>>(buffer) {
                        Ok(deserialized) => {
                            debug!(
                                "Received {:?} packets",
                                deserialized.len()
                            );

                            for packet in deserialized.into_iter() {
                                client
                                    .add_received_message(packet, command_frame)
                            }
                        }
                        Err(e) => {
                            error!("Error occurred when deserializing TCP-packet. Reason: {:?}", e);
                        }
                        // };
                        //     }
                        //     Err(e) => {
                        //         error!("Error occurred when decompressing TCP-packet. Reason: {:?}", e);
                        //     }
                    }
                }
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::ConnectionReset => {
                            *active = false;
                            network_events.enqueue(NetworkEvent::Disconnected(peer_addr, client.client_id()))
                        }
                        io::ErrorKind::WouldBlock => {}
                        _ => {}
                    };

                    break;
                }
            };
        }
    }
}

pub fn tcp_server_sent_system<
    S: SerializationStrategy + 'static,
    C: CompressionStrategy + 'static,
    ServerToClientMessage: NetworkMessage,
    ClientToServerMessage: NetworkMessage,
    ClientToServerCommand: NetworkCommand
>(tcp: &mut TcpListenerResource, postoffice: &mut PostOffice<ServerToClientMessage, ClientToServerMessage, ClientToServerCommand>, packer: &Packer<S, C>, network_events: &mut NetworkEventQueue) {
    for client in postoffice.clients_mut() {
        let addr = client.1.addr();

        let postbox = client.1.postbox_mut();
        let client_stream = tcp
            .get_stream(addr)
            .expect("TCP didn't exist while it is supposed to.");

        let packets = postbox
            .drain_outgoing(|_| true)
            .into_iter()
            .collect::<Vec<transport::ServerToClientMessage<ServerToClientMessage>>>();

        if packets.len() == 0 {
            continue;
        }

        match &packer.serialization().serialize(&packets) {
            Ok(serialized) => {
                debug!(
                    "Sending {} packets to TCP stream.",
                    packets.len()
                );

                // let compressed = packer.compression().compress(&serialized);
                //
                // if let Err(e) = client_stream.1.write(&compressed) {
                //     error!("Error occurred when sending TCP-packet. Reason: {:?}", e);
                // }

                if let Err(e) = client_stream.1.write(&serialized) {
                    match e.kind() {
                        io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe => {
                            network_events.enqueue(NetworkEvent::Disconnected(addr, *client.0))
                        },
                        _ => {
                            error!("Error occurred when sending TCP-packet. Reason: {:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!(
                    "Error occurred when serializing TCP-packet. Reason: {:?}",
                    e
                );
            }
        }
    }
}