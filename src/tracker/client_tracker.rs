use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
};

use serde_diff::{Config, Diff, FieldPathMode};

use crate::{
    error::ErrorKind,
    synchronisation::{CommandFrame, NetworkCommand},
    tracker::{ClientChangeTracker, TrackableMarker},
    uid::Uid,
};

/// Tracks value modifications of a type and sends events with these changes.
///
/// The [Tracker](./struct.Tracker.html) implements [DerefMut](./struct.Tracker.html#impl-DerefMut) which makes it possible to treat this tracker as if you are working with the type you track.
/// On [Drop](./struct.Tracker.html#impl-Drop) it checks if modifications have been made.
/// If this is the case only the modified fields in an event will be sent to the given sender.
pub struct ClientModificationTracker<'borrow, 'notifier, Component, Tracker, Command>
where
    Component: TrackableMarker,
    Tracker: ClientChangeTracker<Command>,
    Command: NetworkCommand,
{
    unchanged: Component,
    borrow: &'borrow mut Component,
    tracker: &'notifier mut Tracker,
    identifier: Uid,
    command_frame: CommandFrame,
    command: Command,
}

impl<'borrow, 'notifier, C, T, CM> ClientModificationTracker<'borrow, 'notifier, C, T, CM>
where
    C: TrackableMarker,
    T: ClientChangeTracker<CM>,
    CM: NetworkCommand,
{
    /// Constructs a new tracker.
    ///
    /// * `borrow`: mutable reference to the object which modifications are tracked.
    /// * `serialization`: an instance of a type that implements [SerializationStrategy](../track/serialization/trait.SerializationStrategy.html) strategy.
    ///     This serializer is needed to monitor the changes and the serialized mutations are sent along with the event.
    pub fn new(
        borrow: &'borrow mut C,
        tracker: &'notifier mut T,
        identifier: Uid,
        command_frame: CommandFrame,
        command: CM,
    ) -> ClientModificationTracker<'borrow, 'notifier, C, T, CM> {
        ClientModificationTracker {
            unchanged: (borrow.deref()).clone(),
            borrow,
            tracker,
            identifier,
            command_frame,
            command,
        }
    }

    pub fn unchanged(&self) -> &C {
        return &self.unchanged;
    }

    pub fn serialize_unchanged(&self) -> Result<Vec<u8>, ErrorKind> {
        bincode::serialize(&self.unchanged)
            .map_err(|e| ErrorKind::SerializationError(e.to_string()))
    }

    pub fn serialize_changed(&self) -> Result<Vec<u8>, ErrorKind> {
        bincode::serialize(&self.borrow).map_err(|e| ErrorKind::SerializationError(e.to_string()))
    }

    fn configure_diff(&self) -> Diff<'_, '_, C> {
        Config::new()
            .with_field_path_mode(FieldPathMode::Index)
            .serializable_diff(&self.unchanged, &self.borrow)
    }
}

impl<'borrow, 'notifier, C, T, CM> Deref for ClientModificationTracker<'borrow, 'notifier, C, T, CM>
where
    C: TrackableMarker,
    T: ClientChangeTracker<CM>,
    CM: NetworkCommand,
{
    type Target = C;

    /// Returns a reference to the underlying type being tracked.
    fn deref(&self) -> &Self::Target {
        &self.borrow
    }
}

impl<'borrow, 'notifier, C, T, CM> DerefMut
    for ClientModificationTracker<'borrow, 'notifier, C, T, CM>
where
    C: TrackableMarker,
    T: ClientChangeTracker<CM>,
    CM: NetworkCommand,
{
    /// Returns a mutable reference to the underlying type being tracked.
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.borrow
    }
}

impl<'borrow, 'notifier, C, T, CM> Drop for ClientModificationTracker<'borrow, 'notifier, C, T, CM>
where
    C: TrackableMarker,
    T: ClientChangeTracker<CM>,
    CM: NetworkCommand,
{
    /// Checks to see if any field values have changed.
    /// If this is the case, the changed fields will be packed into an event and an event will be sent.
    fn drop(&mut self) {
        let diff = self.configure_diff();

        match bincode::serialize(&diff) {
            Ok(_data) => {
                if diff.has_changes() {
                    let unchanged_serialized = self
                        .serialize_unchanged()
                        .expect("Error while serializing unchanged component.");
                    let changed_serialized = self
                        .serialize_changed()
                        .expect("Error while serializing unchanged component.");

                    self.tracker.push(
                        self.command.clone(),
                        self.command_frame,
                        self.identifier,
                        unchanged_serialized,
                        changed_serialized,
                        TypeId::of::<C>(),
                    );
                } else {
                    println!("not different")
                }
            }
            Err(e) => {
                panic!(
                    "Could not serialize modification information because: {:?}",
                    e
                );
            }
        };
    }
}
