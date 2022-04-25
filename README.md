Provides data for other instances on the same machine/network

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
use instance_chart::{discovery, ChartBuilder};

#[tokio::main]
async fn main() {
   let chart = ChartBuilder::new()
       .with_id(1)
       .with_service_port(8042)
       .finish()
       .unwrap();
   let maintain = discovery::maintain(chart.clone());
   let _ = tokio::spawn(maintain); // maintain task will run forever
 }
 ```
