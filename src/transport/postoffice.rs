use std::collections::VecDeque;
use crate::transport::{Message, UrgencyRequirement};
use crate::transport::client::{Clients};
use std::iter::{Enumerate};
use std::collections::vec_deque::{Iter, IterMut};
use crate::transport::PostBoxMessage;
use std::net::SocketAddr;
use std::ops::{DerefMut, Deref};

pub struct InboxEntry<In: PostBoxMessage> {
    pub (crate) acknowledged: bool,
    pub (crate) message: In
}

impl<In: PostBoxMessage> InboxEntry<In> {
    pub fn new(message: In, acknowledged: bool) -> InboxEntry<In> {
        InboxEntry {
            message,
            acknowledged
        }
    }

    pub fn acknowledged(&self) -> bool {
        self.acknowledged
    }

    pub fn set_acknowledged(&mut self, acknowledged: bool) {
        self.acknowledged = acknowledged;
    }
}

impl <In: PostBoxMessage> Deref for InboxEntry<In> {
    type Target = In;

    fn deref(&self) -> &Self::Target {
        &self.message
    }
}

impl <In: PostBoxMessage> DerefMut for InboxEntry<In> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.message
    }
}

pub struct PostBox<In: PostBoxMessage, Out: PostBoxMessage> { // seperate inbox from outbox generic
    addr: SocketAddr,
    inbox: VecDeque<InboxEntry<In>>,
    outgoing: VecDeque<Message<Out>>,
}

impl<In: PostBoxMessage, Out: PostBoxMessage> PostBox<In, Out> {
    pub fn new(addr: SocketAddr) -> PostBox<In, Out> {
        PostBox {
            addr,
            inbox: VecDeque::new(),
            outgoing: VecDeque::new()
        }
    }

    pub fn add_to_inbox(&mut self, event: In) {
        self.inbox
            .push_back(InboxEntry::new(event, false));
    }

    pub fn add_acknowledge_to_inbox(&mut self, event: In) {
        self.inbox
            .push_back(InboxEntry::new(event, true));
    }

    /// Returns true if there are messages enqueued to be sent.
    pub fn empty_inbox(&self) -> bool {
        self.inbox.is_empty()
    }

    /// Returns true if there are messages enqueued to be sent.
    pub fn empty_outgoing(&self) -> bool {
        self.outgoing.is_empty()
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Creates a `Message` with the default guarantees provided by the `Socket` implementation and
    /// pushes it onto the messages queue to be sent on next sim tick.
    pub fn send(&mut self, event: Out) {
        self.outgoing
            .push_back(Message::new(event, UrgencyRequirement::OnTick));
    }

    /// Creates a `Message` with the default guarantees provided by the `Socket` implementation and
    /// Pushes it onto the messages queue to be sent immediately.
    pub fn send_immediate(&mut self, event: Out) {
        self.outgoing
            .push_back(Message::new(event, UrgencyRequirement::Immediate));
    }

    /// Returns a reference to the owned messages.
    pub fn get_outgoing(&self) -> &VecDeque<Message<Out>> {
        &self.outgoing
    }

    /// Returns the messages to send by returning the immediate messages or anything adhering to
    /// the given filter.
    pub fn drain_outgoing_with_priority(
        &mut self,
        mut filter: impl FnMut(&mut Message<Out>) -> bool,
    ) -> Vec<Message<Out>> {
        self.drain_outgoing(|message| {
            message.urgency() == UrgencyRequirement::Immediate || filter(message)
        })
    }

    /// Drains the messages queue and returns the drained messages. The filter allows you to drain
    /// only messages that adhere to your filter. This might be useful in a scenario like draining
    /// messages with a particular urgency requirement.
    pub fn drain_outgoing(&mut self, mut filter: impl FnMut(&mut Message<Out>) -> bool) -> Vec<Message<Out>> {
        let mut drained = Vec::with_capacity(self.outgoing.len());
        let mut i = 0;
        while i != self.outgoing.len() {
            if filter(&mut self.outgoing[i]) {
                if let Some(m) = self.outgoing.remove(i) {
                    drained.push(m);
                }
            } else {
                i += 1;
            }
        }

        drained
    }

    pub fn drain_inbox(&mut self, mut filter: impl FnMut(&In) -> bool) -> Vec<In> {
        let mut drained = Vec::with_capacity(self.inbox.len());
        let mut i = 0;
        while i != self.inbox.len() {
            if self.inbox[i].acknowledged() && filter(&self.inbox[i]) {
                if let Some(entry) = self.inbox.remove(i) {
                    drained.push(entry.message);
                }
            } else {
                i += 1;
            }
        }
        drained
    }

    pub fn remove_from_inbox(&mut self, index: usize) {
        self.inbox.remove(index);
    }

    pub fn enumerate_inbox(&self) -> Enumerate<Iter<InboxEntry<In>>> {
        self.inbox.iter().enumerate()
    }

    pub fn enumerate_inbox_mut(&mut self) -> Enumerate<IterMut<InboxEntry<In>>> {
        self.inbox.iter_mut().enumerate()
    }
}

pub struct PostOffice {
    clients: Clients,
    client_count: u16
}

impl PostOffice {
    pub fn new() -> PostOffice {
        PostOffice {
            clients: Clients::new(),
            client_count: 0
        }
    }

    pub fn register_client(&mut self, addr: SocketAddr) {
        self.clients.add(addr, self.client_count);
        self.client_count += 1;
    }

    pub fn clients_mut(&mut self) -> &mut Clients {
        &mut self.clients
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ComponentRecord, SentPacket};
    use std::net::SocketAddr;

    #[test]
    fn test_send_with_default_requirements() {
        let mut resource = create_test_resource();

        resource.send(remove_event());

        let packet = &resource.messages[0];

        assert_eq!(resource.messages.len(), 1);
        assert_eq!(packet.urgency(), UrgencyRequirement::OnTick);
    }

    #[test]
    fn test_send_immediate_message() {
        let mut resource = create_test_resource();

        resource.send_immediate(modify_event());

        let packet = &resource.messages[0];

        assert_eq!(resource.messages.len(), 1);
        assert_eq!(packet.urgency(), UrgencyRequirement::Immediate);
    }

    #[test]
    fn test_has_messages() {
        let mut resource = create_test_resource();
        assert_eq!(resource.has_messages(), false);
        resource.send_immediate(modify_event());
        assert_eq!(resource.has_messages(), true);
    }

    #[test]
    fn test_drain_only_immediate_messages() {
        let mut resource = create_test_resource();

        let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();
        resource.send_immediate(modify_event());
        resource.send_immediate(modify_event());
        resource.send(remove_event());
        resource.send(remove_event());
        resource.send_immediate(modify_event());

        assert_eq!(resource.drain_messages_to_send(|_| false).len(), 3);
        assert_eq!(resource.drain_messages_to_send(|_| false).len(), 0);
    }

    #[test]
    fn drain_removed_events() {
        let mut buffer = ReceiveBufferResource::default();
        packets().into_iter().for_each(|f| buffer.push(f));

        assert_eq!(buffer.drain_removed().len(), 1);
        assert_eq!(buffer.drain_removed().len(), 0);
    }

    #[test]
    fn drain_inserted_events() {
        let mut buffer = ReceiveBufferResource::default();
        packets().into_iter().for_each(|f| buffer.push(f));

        assert_eq!(buffer.drain_inserted().len(), 2);
        assert_eq!(buffer.drain_inserted().len(), 0);
    }

    #[test]
    fn drain_modified_events() {
        let mut buffer = ReceiveBufferResource::default();
        packets().into_iter().for_each(|f| buffer.push(f));

        // There are three modification events.
        assert_eq!(buffer.drain_modified(Uid(0), Uid(2)).len(), 2);
        assert_eq!(buffer.drain_modified(Uid(0), Uid(1)).len(), 1);

        // Everything should be drained.
        assert_eq!(buffer.drain_modified(Uid(0), Uid(1)).len(), 0);
        assert_eq!(buffer.drain_modified(Uid(0), Uid(2)).len(), 0);
    }

    fn packets() -> Vec<ReceivedPacket> {
        let addr = "127.0.0.1:1234".parse().unwrap();
        let id = Uid(0);

        vec![
            ReceivedPacket::new(addr, SentPacket::new(ClientMessage::EntityRemoved(id))),
            ReceivedPacket::new(addr, SentPacket::new(ClientMessage::EntityInserted(id, vec![]))),
            ReceivedPacket::new(addr, SentPacket::new(ClientMessage::EntityInserted(id, vec![]))),
            ReceivedPacket::new(
                addr,
                SentPacket::new(ClientMessage::ComponentModified(
                    id,
                    ComponentRecord::new(1, vec![]),
                )),
            ),
            ReceivedPacket::new(
                addr,
                SentPacket::new(ClientMessage::ComponentModified(
                    id,
                    ComponentRecord::new(2, vec![]),
                )),
            ),
            ReceivedPacket::new(
                addr,
                SentPacket::new(ClientMessage::ComponentModified(
                    id,
                    ComponentRecord::new(2, vec![]),
                )),
            ),
        ]
    }

    fn modify_event() -> ClientMessage {
        ClientMessage::ComponentModified(Uid(0), ComponentRecord::new(0, test_payload().to_vec()))
    }

    fn remove_event() -> ClientMessage {
        ClientMessage::EntityRemoved(Uid(0))
    }

    fn test_payload() -> &'static [u8] {
        b"test"
    }

    fn create_test_resource() -> SentBufferResource {
        SentBufferResource::new()
    }
}
