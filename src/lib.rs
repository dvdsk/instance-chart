//! Provides data about other instances on the same machine/network
//!
//! This crate provides a lightweight alternative to `mDNS`. It discovers other instances on the
//! same network or (optionally) machine. You provide an `Id` and some `data` you want to share.
//! Usually this is a port your service uses. This gives you a live updating chart
//! of all the discovered `Ids`-`data` pairs and the corrosponding ip adresses.
//!
//! The chart can contain instances that are now down. It can not be used to check if a service is
//! up.
//!
//! ## Usage
//!
//! Add a dependency on `instance-chart` in `Cargo.toml`:
//!
//! ```toml
//! instance_chart = "0.1"
//! ```
//!
//! Now add the following snippet somewhere in your codebase. Discovery will stop when you drop the
//! maintain future.
//!
//! ```rust
//! use std::error::Error;
//! use instance_chart::{discovery, ChartBuilder};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!    let chart = ChartBuilder::new()
//!        .with_id(1)
//!        .with_service_port(8042)
//!        .finish()?;
//!    let maintain = discovery::maintain(chart.clone());
//!    let _ = tokio::spawn(maintain); // maintain task will run forever
//!    Ok(())
//! }
//! ```

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
    /// Error binding to socket, you might want to try another discovery port and/or enable local_discovery.
    #[error("Error binding to socket, you might want to try another discovery port and/or enable local_discovery.")]
    Bind { error: io::Error, port: u16 },
    /// Failed joining multicast network
    #[error("Failed joining multicast network")]
    JoinMulticast(io::Error),
    /// Failed to transform blocking to async socket
    #[error("Failed to transform blocking to async socket")]
    ToTokio(io::Error),
}
