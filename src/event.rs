use crate::transport::ClientId;
use std::{collections::VecDeque, net::SocketAddr};

pub enum NetworkEvent {
    Connected(SocketAddr),
    Disconnected(SocketAddr, ClientId),
}

pub struct NetworkEventQueue {
    network_events: VecDeque<NetworkEvent>,
}

impl NetworkEventQueue {
    pub fn new() -> NetworkEventQueue {
        NetworkEventQueue {
            network_events: VecDeque::new(),
        }
    }

    pub fn dequeue(&mut self) -> Option<NetworkEvent> {
        self.network_events.pop_front()
    }

    pub fn peek(&self) -> Option<&NetworkEvent> {
        self.network_events.get(0)
    }

    pub fn enqueue(&mut self, event: NetworkEvent) {
        self.network_events.push_back(event)
    }
}
