pub mod apply;
pub mod clock;
pub mod compression;
pub mod error;
pub mod packer;
pub mod serialization;
pub mod synchronisation;
pub mod tracker;
pub mod transport;
pub mod uid;
pub mod event;

pub mod track_attr {
    //! This module can be used to import all types that are required by the [track](./struct.Tracker.html) attribute.

    pub use serde::{Deserialize, Serialize};

    pub use track_macro::track;

    pub use crate::{
        re_exports::serde_diff::{self, *},
        serialization::{bincode::Bincode, SerializationStrategy},
        synchronisation::{CommandFrame, NetworkCommand},
        tracker::{
            ClientChangeTracker, ClientModificationTracker, ServerChangeTracker,
            ServerModificationTracker, Trackable, TrackableMarker,
        },
        uid::Uid,
    };
}

pub mod re_exports {
    //! This module exposes access to the dependencies of this crate.

    /// A re-export of the [serde](https://crates.io/crates/serde) create.
    pub mod serde {
        pub use serde::*;
    }

    /// A re-export of the [serde-diff](https://crates.io/crates/serde-diff) create.
    pub mod serde_diff {
        pub use serde_diff::*;
    }

    /// A re-export of the [bit-set](https://crates.io/crates/bit-set) create.
    pub mod bit_set {
        pub use bit_set::*;
    }
}
