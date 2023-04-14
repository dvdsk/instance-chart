#![doc= include_str!("../README.md")]

mod chart;
pub mod discovery;
mod util;
use std::io;

pub use chart::{Chart, ChartBuilder, Notify};

/// Identifier for a single instance of `Chart`. Must be unique.
pub type Id = u64;

/// Errors that can occure while building a Chart. Except for [`Bind`](Error::Bind) these rarely
/// occur.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Could not set up bare socket
    #[error("Could not set up bare socket")]
    Construct(io::Error),
    /// Failed to set Reuse flag on the socket
    #[error("Failed to set Reuse flag on the socket")]
    SetReuse(io::Error),
    /// Failed to set Broadcast flag on the socket
    #[error("Failed to set Broadcast flag on the socket")]
    SetBroadcast(io::Error),
    /// Failed to set Multicast flag on the socket
    #[error("Failed to set Multicast flag on the socket")]
    SetMulticast(io::Error),
    /// Failed to set TTL flag on the socket
    #[error("Failed to set TTL flag on the socket")]
    SetTTL(io::Error),
    /// Failed to set NonBlocking flag on the socket
    #[error("Failed to set NonBlocking flag on the socket")]
    SetNonBlocking(io::Error),
    /// Error binding to socket, you might want to try another discovery port and/or enable [`local_discovery`](ChartBuilder::local_discovery).
    #[error("Error binding to socket, you might want to try another discovery port and/or enable local_discovery.")]
    Bind { error: io::Error, port: u16 },
    /// Failed joining multicast network
    #[error("Failed joining multicast network")]
    JoinMulticast(io::Error),
    /// Failed to transform blocking to async socket
    #[error("Failed to transform blocking to async socket")]
    ToTokio(io::Error),
}
