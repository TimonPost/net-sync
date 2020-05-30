use std::hash::Hash;

use serde::{Deserialize, Serialize};

pub use self::{
    client::{Client, ClientId},
    message::*,
    postbox::PostBox,
    postoffice::PostOffice,
};

mod client;
mod message;
mod postbox;
mod postoffice;
pub mod tcp;

pub trait NetworkMessage:
    Serialize + for<'a> Deserialize<'a> + Send + Sync + Clone + 'static
{
}

pub trait NetworkCommand: Clone + NetworkMessage + Hash + Eq + PartialEq + 'static {}
