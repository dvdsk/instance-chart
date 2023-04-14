use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::Error;

use super::{interval, Chart, Id};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::Serialize;
use tokio::net::UdpSocket;
use tokio::sync::broadcast;
use tracing::info;

#[derive(Debug, Default)]
pub struct Yes;
#[derive(Debug, Default)]
pub struct No;

pub trait ToAssign: core::fmt::Debug {}
pub trait Assigned: ToAssign {}
pub trait NotAssigned: ToAssign {}

impl ToAssign for Yes {}
impl ToAssign for No {}

impl Assigned for Yes {}
impl NotAssigned for No {}

const DEFAULT_HEADER: u64 = 6_687_164_552_036_412_667;
const DEFAULT_PORT: u16 = 8080;

pub type Port = u16;

/// Construct a Chart using a builder-like pattern. You must always set an `id`. You also
/// need to set [`service port`](ChartBuilder::with_service_port) or [`service ports`](ChartBuilder::with_service_ports). Now you can build with [`finish`](ChartBuilder::finish) or using [`custom_msg`][ChartBuilder::custom_msg]. The latter allowes you to set a custom message to share with other instances when they discover you.
#[allow(clippy::pedantic)]
pub struct ChartBuilder<const N: usize, IdSet, PortSet, PortsSet>
where
    IdSet: ToAssign,
    PortSet: ToAssign,
    PortsSet: ToAssign,
{
    header: u64,
    service_id: Option<Id>,
    discovery_port: u16,
    service_port: Option<u16>,
    service_ports: [u16; N],
    rampdown: interval::Params,
    local: bool,
    id_set: PhantomData<IdSet>,
    port_set: PhantomData<PortSet>,
    ports_set: PhantomData<PortsSet>,
}

impl<const N: usize> ChartBuilder<N, No, No, No> {
    /// Create a new chart builder
    #[allow(clippy::new_without_default)] // builder struct not valid without other methods
    #[must_use]
    pub fn new() -> ChartBuilder<N, No, No, No> {
        ChartBuilder {
            header: DEFAULT_HEADER,
            service_id: None,
            discovery_port: DEFAULT_PORT,
            service_ports: [0u16; N],
            service_port: None,
            rampdown: interval::Params::default(),
            local: false,
            id_set: PhantomData {},
            port_set: PhantomData {},
            ports_set: PhantomData {},
        }
    }
}

impl<const N: usize, IdSet, PortSet, PortsSet> ChartBuilder<N, IdSet, PortSet, PortsSet>
where
    IdSet: ToAssign,
    PortSet: ToAssign,
    PortsSet: ToAssign,
{
    /// Set the [`Id`] for this node, the [`Id`] is the key for this node in the chart
    /// # Note
    /// Always needed, you can not build without an [`Id`] set. The [`Id`] must be __unique__
    #[must_use]
    pub fn with_id(self, id: Id) -> ChartBuilder<N, Yes, PortSet, PortsSet> {
        ChartBuilder {
            header: self.header,
            discovery_port: self.discovery_port,
            service_id: Some(id),
            service_port: self.service_port,
            service_ports: self.service_ports,
            rampdown: self.rampdown,
            local: self.local,
            id_set: PhantomData {},
            port_set: PhantomData {},
            ports_set: PhantomData {},
        }
    }

    /// Use a true random number from a reliable source of randomness as an [`Id`].
    /// # Note
    /// I recommend setting the ['Id'] in a deterministic way if possible, it makes debugging a lot
    /// easier. Theoretically using this method can fail if multiple instances get the same random
    /// number, the chance of this is unrealistically small.
    ///
    /// It is *extreemly* unlikely though possible that this fails. This happens if the systems source of random is configured incorrectly.
    #[must_use]
    pub fn with_random_id(self) -> ChartBuilder<N, Yes, PortSet, PortsSet> {
        let mut rng = OsRng::default();
        let id = rng.next_u64();
        info!("Using random id: {id}");
        ChartBuilder {
            header: self.header,
            discovery_port: self.discovery_port,
            service_id: Some(id),
            service_port: self.service_port,
            service_ports: self.service_ports,
            rampdown: self.rampdown,
            local: self.local,
            id_set: PhantomData {},
            port_set: PhantomData {},
            ports_set: PhantomData {},
        }
    }
    /// Set a `port` for use by your application. This will appear to the other
    /// nodes in the Chart.
    /// # Note
    /// You need to use this or [`with_service_ports`](Self::with_service_ports) if
    /// building with [`finish()`](Self::finish). Cant be used when bulding with
    /// [`custom_msg`](Self::custom_msg).
    #[must_use]
    pub fn with_service_port(self, port: u16) -> ChartBuilder<N, IdSet, Yes, No> {
        ChartBuilder {
            header: self.header,
            discovery_port: self.discovery_port,
            service_id: self.service_id,
            service_port: Some(port),
            service_ports: self.service_ports,
            rampdown: self.rampdown,
            local: self.local,
            id_set: PhantomData {},
            port_set: PhantomData {},
            ports_set: PhantomData {},
        }
    }
    /// Set mutiple `ports` for use by your application. This will appear to the other
    /// nodes in the Chart.
    /// # Note
    /// You need to use this or [`with_service_port`](Self::with_service_port) if
    /// building with [`finish()`](Self::finish). Cant be used when bulding with
    /// [`custom_msg`](Self::custom_msg).
    #[must_use]
    pub fn with_service_ports(self, ports: [u16; N]) -> ChartBuilder<N, IdSet, No, Yes> {
        ChartBuilder {
            header: self.header,
            discovery_port: self.discovery_port,
            service_id: self.service_id,
            service_port: None,
            service_ports: ports,
            rampdown: self.rampdown,
            local: self.local,
            id_set: PhantomData {},
            port_set: PhantomData {},
            ports_set: PhantomData {},
        }
    }
    /// set a custom header number. The header is used to identify your application's chart
    /// from others multicast traffic when deployed your should set this to a [random](https://www.random.org) number.
    #[must_use]
    pub fn with_header(mut self, header: u64) -> ChartBuilder<N, IdSet, PortSet, PortsSet> {
        self.header = header;
        self
    }
    /// set custom port for discovery. With [local discovery] enabled this port needs to be
    /// free and unused on all nodes it is not free the multicast traffic caused by this library
    /// might corrupt network data of other applications. The default port is 8080.
    /// # Warning
    /// Not all ports seem to pass multicast traffic, you might need to experiment a bit.
    #[must_use]
    pub fn with_discovery_port(mut self, port: u16) -> ChartBuilder<N, IdSet, PortSet, PortsSet> {
        self.discovery_port = port;
        self
    }
    /// set duration between discovery broadcasts, decreases linearly from `max` to `min`
    /// over `rampdown` period.
    /// # Panics
    /// panics if min is larger then max
    #[must_use]
    pub fn with_rampdown(
        mut self,
        min: Duration,
        max: Duration,
        rampdown: Duration,
    ) -> ChartBuilder<N, IdSet, PortSet, PortsSet> {
        assert!(
            min <= max,
            "minimum duration: {min:?} must be smaller or equal to the maximum: {max:?}"
        );
        self.rampdown = interval::Params { rampdown, min, max };
        self
    }

    #[must_use]
    /// set whether discovery is enabled within the same host. Defaults to false.
    ///
    /// # Warning
    /// When this is enabled you might not be warned if the `discovery port` is in use by another application.
    /// The other application will recieve network traffic from this crate. This might lead to
    /// corruption in the application if it can not handle this.
    /// `ChartBuilder` will still fail if the `discovery port` is already bound to a multicast adress
    /// without `SO_REUSEADDR` set.
    pub fn local_discovery(
        mut self,
        is_enabled: bool,
    ) -> ChartBuilder<N, IdSet, PortSet, PortsSet> {
        self.local = is_enabled;
        self
    }
}

impl ChartBuilder<1, Yes, No, No> {
    /// build a chart with a custom msg instead of a service port. The message can
    /// be any struct that implements `Debug`, `Clone`, `serde::Serialize` and `serde::Deserialize`
    ///
    /// # Errors
    /// This errors if the discovery port could not be opened. see: [`Self::with_discovery_port`].
    ///
    /// # Example
    /// ```rust
    ///use instance_chart::{discovery, ChartBuilder};
    ///use serde::{Serialize, Deserialize};
    ///use std::error::Error;
    ///use std::time::Duration;
    ///
    ///#[derive(Debug, Clone, Serialize, Deserialize)]
    ///struct Msg(u32);
    ///
    ///#[tokio::main]
    ///async fn main() -> Result<(), Box<dyn Error>> {
    ///   let msg = Msg(0);
    ///   let chart = ChartBuilder::new()
    ///       .with_id(1)
    ///       .with_discovery_port(8888)
    ///       .with_header(17249479) // optional
    ///       .with_rampdown( // optional
    ///           Duration::from_millis(10),
    ///           Duration::from_secs(10),
    ///           Duration::from_secs(60))
    ///       .custom_msg(msg)?;
    ///   let maintain = discovery::maintain(chart.clone());
    ///   let _ = tokio::spawn(maintain); // maintain task will run forever
    ///   Ok(())
    /// }
    /// ```
    #[allow(clippy::missing_panics_doc)] // with generic IdSet and PortSet set service_id must be set
    pub fn custom_msg<Msg>(self, msg: Msg) -> Result<Chart<1, Msg>, Error>
    where
        Msg: Debug + Serialize + Clone,
    {
        let sock = open_socket(self.discovery_port, self.local)?;
        Ok(Chart {
            header: self.header,
            service_id: self.service_id.unwrap(),
            msg: [msg],
            sock: Arc::new(sock),
            map: Arc::new(Mutex::new(HashMap::new())),
            interval: self.rampdown.into(),
            broadcast: broadcast::channel(u8::MAX as usize).0,
        })
    }
}

impl ChartBuilder<1, Yes, Yes, No> {
    /// build a chart that has a single service ports set
    ///
    /// # Errors
    /// This errors if the discovery port could not be opened. see: [`Self::with_discovery_port`].
    ///
    /// # Example
    /// ```rust
    ///use std::error::Error;
    ///use instance_chart::{discovery, ChartBuilder};
    ///use std::time::Duration;
    ///
    ///#[tokio::main]
    ///async fn main() -> Result<(), Box<dyn Error>> {
    ///   let chart = ChartBuilder::new()
    ///       .with_id(1)
    ///       .with_service_port(8042)
    ///       .with_discovery_port(8888)
    ///       .with_header(17249479) // optional
    ///       .with_rampdown( // optional
    ///           Duration::from_millis(10),
    ///           Duration::from_secs(10),
    ///           Duration::from_secs(60))
    ///       .finish()?;
    ///   let maintain = discovery::maintain(chart.clone());
    ///   let _ = tokio::spawn(maintain); // maintain task will run forever
    ///   Ok(())
    /// }
    /// ```

    // with generic IdSet, PortSet set service_id and service_port are always Some
    #[allow(clippy::missing_panics_doc)]
    pub fn finish(self) -> Result<Chart<1, Port>, Error> {
        let sock = open_socket(self.discovery_port, self.local)?;
        Ok(Chart {
            header: self.header,
            service_id: self.service_id.unwrap(),
            msg: [self.service_port.unwrap()],
            sock: Arc::new(sock),
            map: Arc::new(Mutex::new(HashMap::new())),
            interval: self.rampdown.into(),
            broadcast: broadcast::channel(u8::MAX as usize).0,
        })
    }
}

impl<const N: usize> ChartBuilder<N, Yes, No, Yes> {
    /// build a chart that has a multiple service ports set
    ///
    /// # Errors
    /// This errors if the discovery port could not be opened. see: [`Self::with_discovery_port`].
    ///
    /// # Example
    /// ```rust
    ///use std::error::Error;
    ///use instance_chart::{discovery, ChartBuilder};
    ///use std::time::Duration;
    ///
    ///#[tokio::main]
    ///async fn main() -> Result<(), Box<dyn Error>> {
    ///   let chart = ChartBuilder::new()
    ///       .with_id(1)
    ///       .with_service_ports([8042,9042])
    ///       .with_discovery_port(8888)
    ///       .with_header(17249479) // optional
    ///       .with_rampdown( // optional
    ///           Duration::from_millis(10),
    ///           Duration::from_secs(10),
    ///           Duration::from_secs(60))
    ///       .finish()?;
    ///   let maintain = discovery::maintain(chart.clone());
    ///   let _ = tokio::spawn(maintain); // maintain task will run forever
    ///   Ok(())
    /// }
    /// ```

    // with generic IdSet, PortSets set service_id and service_ports are always Some
    #[allow(clippy::missing_panics_doc)]
    pub fn finish(self) -> Result<Chart<N, Port>, Error> {
        let sock = open_socket(self.discovery_port, self.local)?;
        Ok(Chart {
            header: self.header,
            service_id: self.service_id.unwrap(),
            msg: self.service_ports,
            sock: Arc::new(sock),
            map: Arc::new(Mutex::new(HashMap::new())),
            interval: self.rampdown.into(),
            broadcast: broadcast::channel(u8::MAX as usize).0,
        })
    }
}

fn open_socket(port: u16, local_discovery: bool) -> Result<UdpSocket, Error> {
    use socket2::{Domain, SockAddr, Socket, Type};
    use Error::{
        Bind, Construct, JoinMulticast, SetBroadcast, SetMulticast, SetNonBlocking, SetReuse,
        SetTTL, ToTokio,
    };

    assert_ne!(port, 0);

    let interface = Ipv4Addr::from([0, 0, 0, 0]);
    let multiaddr = Ipv4Addr::from([224, 0, 0, 251]);

    let sock = Socket::new(Domain::IPV4, Type::DGRAM, None).map_err(Construct)?;

    if local_discovery {
        sock.set_reuse_port(true).map_err(SetReuse)?; // allow binding to a port already in use
    }
    sock.set_broadcast(true).map_err(SetBroadcast)?; // enable udp broadcasting
    sock.set_multicast_loop_v4(true).map_err(SetMulticast)?; // send broadcast to self
    sock.set_ttl(4).map_err(SetTTL)?; // deliver to other subnetworks

    let address = SocketAddr::from((interface, port));
    let address = SockAddr::from(address);
    sock.bind(&address).map_err(|error| Bind { error, port })?;
    sock.join_multicast_v4(&multiaddr, &interface)
        .map_err(JoinMulticast)?;

    let sock = std::net::UdpSocket::from(sock);
    sock.set_nonblocking(true).map_err(SetNonBlocking)?;
    let sock = UdpSocket::from_std(sock).map_err(ToTokio)?;
    Ok(sock)
}

#[cfg(test)]
mod compiles {
    use super::*;

    #[tokio::test]
    async fn with_service_port() {
        let chart = ChartBuilder::new()
            .with_id(0)
            .with_service_port(15)
            .local_discovery(true)
            .finish()
            .unwrap();
        let _ = chart.our_service_port();
    }

    #[tokio::test]
    async fn with_service_ports() {
        let chart = ChartBuilder::new()
            .with_id(0)
            .with_service_ports([1, 2])
            .local_discovery(true)
            .finish()
            .unwrap();
        let _ = chart.our_service_ports();
    }

    #[tokio::test]
    async fn custom_msg() {
        let chart = ChartBuilder::new()
            .with_id(0)
            .local_discovery(true)
            .custom_msg("hi")
            .unwrap();
        let _ = chart.our_msg();
    }
}
