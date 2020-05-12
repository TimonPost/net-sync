pub use self::{
    client::{Client, ClientId},
    message::*,
    postbox::PostBox,
    postoffice::PostOffice,
};
use serde::{Serialize, Deserialize};
use std::hash::Hash;

mod client;
mod message;
mod postbox;
mod postoffice;
pub mod tcp;

pub trait NetworkMessage: Serialize + for<'a> Deserialize<'a> + Send + Sync + Clone + 'static {

}

pub trait NetworkCommand: Clone + NetworkMessage + Hash + Eq + PartialEq + 'static {

}