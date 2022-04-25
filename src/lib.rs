//! Provides data for other instances on the same machine/network
//!
//! This crate provides a lightweight alternative to `mDNS`. It discovers other instances on the
//! same network or (optionally) machine. You provide an `Id` and some `data` you want to share.
//! Usually this is a port your service uses. This gives you a live updating chart
//! of all the discovered `Ids`-`data` pairs and the corrosponding ip adresses.
//!
//! The chart can contain instances that are now down. It can not be used to check if a service is
//! up.
//!
//! ## Issues
//! We use UDP multicasts to discover other entries. On most systems only a few ports do not have
//! multicast blocked by firewall. If things are not working you can try setting another port for
//! discovery using: [ChartBuilder::with_discovery_port()].
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
type Id = u64;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error setting up bare socket")]
    Construct(io::Error),
    #[error("Error not set Reuse flag on the socket")]
    SetReuse(io::Error),
    #[error("Error not set Broadcast flag on the socket")]
    SetBroadcast(io::Error),
    #[error("Error not set Multicast flag on the socket")]
    SetMulticast(io::Error),
    #[error("Error not set TTL flag on the socket")]
    SetTTL(io::Error),
    #[error("Error not set NonBlocking flag on the socket")]
    SetNonBlocking(io::Error),
    #[error("Error binding to socket, you might want to try another discovery port and/or enable local_discovery.")]
    Bind { error: io::Error, port: u16 },
    #[error("Error joining multicast network")]
    JoinMulticast(io::Error),
    #[error("Error transforming to async socket")]
    ToTokio(io::Error),
}
