use serde::{Deserialize, Serialize};

pub use track_macro::track;

pub mod apply;
pub mod clock;
pub mod compression;
pub mod error;
pub mod packer;
pub mod serialization;
pub mod state;
pub mod synchronisation;
pub mod tracker;
pub mod transport;
pub mod uid;

pub type EntityId = u32;
pub type ComponentId = u32;

#[derive(Clone, Hash, Debug, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentData(ComponentId, Vec<u8>);

impl ComponentData {
    pub fn new(register_id: u32, data: Vec<u8>) -> ComponentData {
        ComponentData(register_id, data)
    }

    pub fn component_id(&self) -> ComponentId {
        self.0
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.1
    }
}

/// A re-export with types needed for the [track](./struct.Tracker.html) attribute.
pub mod preclude {
    pub use serde::{Deserialize, Serialize};

    pub use track_macro::track;

    // [serde-diff](https://crates.io/crates/serde-diff)s macro's require `serde_diff` to be imported when we use `track` attribute macro.
    pub use crate::tracker::{
        ClientChangeTracker, ClientModificationTracker, ServerChangeTracker,
        ServerModificationTracker, Trackable, TrackableMarker,
    };
    pub use crate::{
        re_exports::serde_diff::{self, *},
        serialization::{bincode::Bincode, SerializationStrategy},
        synchronisation::CommandFrame,
        transport::NetworkCommand,
        uid::Uid,
    };
}

pub mod re_exports {
    /// A re-export of the [serde](https://crates.io/crates/serde) create.
    pub mod serde {
        pub use serde::*;
    }

    /// A re-export of the [serde-diff](https://crates.io/crates/serde-diff) create.
    pub mod serde_diff {
        pub use serde_diff::*;
    }

    /// A re-export of the [crossbeam-channel](https://crates.io/crates/crossbeam-channel) create.
    pub mod crossbeam_channel {
        pub use crossbeam_channel::*;
    }
}
