//! This module provides code for tracking state changes.

use std::{any::TypeId, fmt::Debug};

use serde::Serialize;
use serde_diff::SerdeDiff;

pub use client_tracker::ClientModificationTracker;
pub use server_tracker::ServerModificationTracker;
pub use track::TrackResource;

use crate::{
    serialization,
    synchronisation::{CommandFrame, NetworkCommand},
    uid::Uid,
};

mod client_tracker;
mod server_tracker;
mod track;

/// A trait with functions for tracking struct value modifications.
///
/// Do not implement this trait manually but use the `track` attribute for less boiler plate code.
pub trait Trackable<Component, Serializer>
where
    Component: TrackableMarker,
    Serializer: serialization::SerializationStrategy,
{
    fn server_track<'notifier, Tracker: ServerChangeTracker>(
        &mut self,
        tracker: &'notifier mut Tracker,
        entity_id: Uid,
        command_frame: CommandFrame,
    ) -> ServerModificationTracker<'_, 'notifier, Component, Serializer, Tracker>;

    fn client_track<'notifier, Tracker: ClientChangeTracker<Command>, Command: NetworkCommand>(
        &mut self,
        tracker: &'notifier mut Tracker,
        command: Command,
        entity_id: Uid,
        command_frame: CommandFrame,
    ) -> ClientModificationTracker<'_, 'notifier, Component, Serializer, Tracker, Command>;
}

/// A marker trait with a number of requirements that are mandatory for trackable types.
pub trait TrackableMarker: Clone + SerdeDiff + Serialize + Debug + Send + Sync + 'static {}

// server push/client push/ voor tracker
pub trait ServerChangeTracker {
    fn push(
        &mut self,
        command_frame: CommandFrame,
        entity_id: Uid,
        unchanged_serialized: Vec<u8>,
        component_type: TypeId,
    );
}

pub trait ClientChangeTracker<C: NetworkCommand> {
    fn push(
        &mut self,
        command: C,
        command_frame: CommandFrame,
        entity_id: Uid,
        unchanged_serialized: Vec<u8>,
        changed_serialized: Vec<u8>,
        component_type: TypeId,
    );
}
