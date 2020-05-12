use std::collections::vec_deque::{Iter, IterMut};
use std::collections::VecDeque;
use std::iter::Enumerate;

use crate::transport::NetworkMessage;

pub struct PostBox<In, Out>
where
    In: NetworkMessage,
    Out: NetworkMessage,
{
    inbox: VecDeque<In>,
    outgoing: VecDeque<Out>,
}

impl<In, Out> PostBox<In, Out>
where
    In: NetworkMessage,
    Out: NetworkMessage,
{
    pub fn new() -> PostBox<In, Out> {
        PostBox {
            inbox: VecDeque::new(),
            outgoing: VecDeque::new(),
        }
    }

    pub fn add_to_inbox(&mut self, event: In) {
        self.inbox.push_back(event);
    }

    /// Returns true if there are messages enqueued to be sent.
    pub fn empty_inbox(&self) -> bool {
        self.inbox.is_empty()
    }

    /// Returns true if there are messages enqueued to be sent.
    pub fn empty_outgoing(&self) -> bool {
        self.outgoing.is_empty()
    }

    /// Creates a `Message` with the default guarantees provided by the `Socket` implementation and
    /// pushes it onto the messages queue to be sent on next sim tick.
    pub fn send(&mut self, event: Out) {
        self.outgoing.push_back(event);
    }

    /// Returns a reference to the owned messages.
    pub fn get_outgoing(&self) -> &VecDeque<Out> {
        &self.outgoing
    }

    /// Drains the messages queue and returns the drained messages. The filter allows you to drain
    /// only messages that adhere to your filter. This might be useful in a scenario like draining
    /// messages with a particular urgency requirement.
    pub fn drain_outgoing(&mut self, mut filter: impl FnMut(&mut Out) -> bool) -> Vec<Out> {
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
            if filter(&self.inbox[i]) {
                if let Some(entry) = self.inbox.remove(i) {
                    drained.push(entry);
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

    pub fn enumerate_inbox(&self) -> Enumerate<Iter<In>> {
        self.inbox.iter().enumerate()
    }

    pub fn enumerate_inbox_mut(&mut self) -> Enumerate<IterMut<In>> {
        self.inbox.iter_mut().enumerate()
    }
}
