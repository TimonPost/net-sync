use crate::uid::Uid;
use serde::{Serialize, Deserialize};
use crate::ComponentData;
use crate::state::WorldState;
use crate::transport::PostBoxMessage;

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientMessage {
    EntityInserted(Uid, Vec<ComponentData>),
    EntityRemoved(Uid),
    ComponentModified(Uid, ComponentData),
    ComponentRemoved(Uid),
    ComponentAdd(Uid, ComponentData),
}

impl PostBoxMessage for ClientMessage { }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMessage {
    StateUpdate(WorldState),
}

impl PostBoxMessage for ServerMessage { }