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
 instance_chart = "0.3"
 ```

 Now add the following snippet somewhere in your codebase. Discovery will stop when you drop the
 maintain future.

 ```rust
 use std::error::Error;
 use instance_chart::{discovery, ChartBuilder};

 #[tokio::main]
 async fn main() -> Result<(), Box<dyn Error>> {
    let chart = ChartBuilder::new()
        .with_id(1) // pick a unique id for each service
        .with_service_port(8042) // The port your service, not discovery, runs on
        .finish()?;
    let maintain = discovery::maintain(chart.clone());
    let _ = tokio::spawn(maintain); // maintain task will run forever
    Ok(())
 }
 ```

## Example App

A minimal peer to peer chat app using instance-chart to discover peers. After type a line the message is sent to all the peers simply by looping over their addresses returned by [`chart.addr_vec()`](Chart::addr_vec).

```rust,ignore
#[tokio::main]
async fn main() {
	let listener = TcpListener::bind(("0.0.0.0", 0)).await.unwrap();
	let port = listener.local_addr().unwrap().port();

	let chart = ChartBuilder::new()
		.with_random_id()
		.with_service_port(port)
		.local_discovery(true)
		.finish()
		.unwrap();

	// run these task forever in the background
	let chart2 = chart.clone();
	tokio::spawn(async move { discovery::maintain(chart2).await });
	tokio::spawn(async move { print_incoming(listener).await });

	spawn_blocking(move || {
		let reader = std::io::stdin();
		loop {
			let mut line = String::new();
			reader.read_line(&mut line).unwrap();
			for (_, addr) in chart.addr_vec() {
				let mut peer = std::net::TcpStream::connect(addr).unwrap();
				peer.write_all(line.as_bytes()).unwrap();
			}
		}
	})
	.await
	.unwrap();
}
```

For the full working example including the `print_incoming` function see [examples/chat.rs](examples/chat.rs).
