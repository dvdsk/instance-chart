Simple lightweight local service discovery for testing

This crate provides a lightweight alternative to `mDNS`. It discovers other instances on the
same machine or network. You provide an Id and Port you wish to be contacted on. Multicast-discovery
then gives you a live updating chart of all the discovered Ids, Ports pairs and their adress.

## Usage

Add a dependency on `multicast-discovery` in `Cargo.toml`:

```toml
multicast-discovery = "0.1"
```

Now add the following snippet somewhere in your codebase. Discovery will stop when you drop the
maintain future.

```rust
se multicast_discovery::{discovery, ChartBuilder};

[tokio::main]
sync fn main() {
  let chart = ChartBuilder::new()
      .with_id(1)
      .with_service_port(8042)
      .finish()
      .unwrap();
  let maintain = discovery::maintain(chart.clone());
  let _ = tokio::spawn(maintain); // maintain task will run forever
}
```
