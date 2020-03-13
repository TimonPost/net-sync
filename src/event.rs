use crate::uid::Uid;
use crate::transport::ComponentRecord;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    EntityInserted(Uid, Vec<ComponentRecord>),
    EntityRemoved(Uid),
    ComponentModified(Uid, ComponentRecord),
    ComponentRemoved(Uid),
    ComponentAdd(Uid, ComponentRecord),
}
