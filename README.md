# Instance chart

 > **Provides data about other instances on the same machine/network**

[![Crates.io](https://img.shields.io/crates/v/instance-chart?style=flat-square)](https://crates.io/crates/instance-chart)
[![Crates.io](https://img.shields.io/crates/d/instance-chart?style=flat-square)](https://crates.io/crates/instance-chart)
[![Docs.rs](https://img.shields.io/docsrs/instance-chart?style=flat-square)](https://docs.rs/instance-chart)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE-MIT)

See also:
 - [API documentation](https://docs.rs/instance-chart)
 - [Changelog](CHANGELOG.md)

 This crate provides a lightweight alternative to `mDNS`. It discovers other instances on the
 same network or (optionally) machine. You provide an `Id` and some `data` you want to share.
 Usually this is a port your service uses. This gives you a live updating chart
 of all the discovered `Ids`-`data` pairs and the corrosponding ip adresses.

 The chart can contain instances that are now down. It can not be used to check if a service is
 up.

 ## Usage

 Add a dependency on `instance-chart` in `Cargo.toml`:

 ```toml
 instance_chart = "0.1"
 ```

 Now add the following snippet somewhere in your codebase. Discovery will stop when you drop the
 maintain future.

 ```rust
 use std::error::Error;
 use instance_chart::{discovery, ChartBuilder};

 #[tokio::main]
 async fn main() -> Result<(), Box<dyn Error>> {
    let chart = ChartBuilder::new()
        .with_id(1)
        .with_service_port(8042)
        .finish()?;
    let maintain = discovery::maintain(chart.clone());
    let _ = tokio::spawn(maintain); // maintain task will run forever
    Ok(())
 }
 ```
