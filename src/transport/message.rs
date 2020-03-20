use serde::{Deserialize, Serialize};
use crate::transport::UrgencyRequirement;

/// Structure used to hold message payloads before they are consumed and sent by an underlying
/// NetworkSystem.
#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message<T> {
    /// The event that defines what kind of packet this is.
    event: T,
    /// The requirement around when this message should be sent.
    urgency: UrgencyRequirement,
}

impl<T> Message<T> {
    /// Creates and returns a new Message.
    pub(crate) fn new(event: T, urgency: UrgencyRequirement) -> Self {
        Self { event, urgency }
    }

    pub fn event(self) -> T {
        self.event
    }

    pub fn event_ref(&self) -> &T {
        &self.event
    }

    pub fn urgency(&self) -> UrgencyRequirement {
        self.urgency
    }
}

#[cfg(test)]
pub mod test {
    use crate::{ClientMessage};
    use crate::transport::{UrgencyRequirement, Message};
    use crate::uid::Uid;

    #[test]
    fn create_message_test() {
        let id = Uid(0);
        let event = ClientMessage::EntityRemoved(Uid(1));
        let requirement = UrgencyRequirement::Immediate;

        let message = Message::new(event.clone(), requirement.clone());
        assert_eq!(message.event_ref(), &event);
        assert_eq!(message.urgency(), requirement);
    }
}
