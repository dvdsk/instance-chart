use std::fmt::Debug;
use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use crate::Error;

use super::{interval, Chart, Id};
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Port(pub u16);
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ports(pub Vec<u16>);

#[derive(Default)]
pub struct ChartBuilder<IdSet, PortSet, PortsSet>
where
    IdSet: ToAssign,
    PortSet: ToAssign,
    PortsSet: ToAssign,
{
    header: u64,
    service_id: Option<Id>,
    discovery_port: u16,       // port the node is listening on for work
    service_port: Option<u16>, // port the node is listening on for work
    service_ports: Vec<u16>,   // port the node is listening on for work
    rampdown: interval::Params,
    id_set: PhantomData<IdSet>,
    port_set: PhantomData<PortSet>,
    ports_set: PhantomData<PortsSet>,
}

impl ChartBuilder<No, No, No> {
    #[must_use]
    pub fn new() -> ChartBuilder<No, No, No> {
        ChartBuilder {
            header: DEFAULT_HEADER,
            service_id: None,
            discovery_port: DEFAULT_PORT,
            service_ports: Vec::new(),
            service_port: None,
            rampdown: interval::Params::default(),
            id_set: PhantomData {},
            port_set: PhantomData {},
            ports_set: PhantomData {},
        }
    }
}

impl<IdSet, PortSet, PortsSet> ChartBuilder<IdSet, PortSet, PortsSet>
where
    IdSet: ToAssign,
    PortSet: ToAssign,
    PortsSet: ToAssign,
{
    #[must_use]
    pub fn with_id(self, id: Id) -> ChartBuilder<Yes, PortSet, PortsSet> {
        ChartBuilder {
            header: self.header,
            discovery_port: self.discovery_port,
            service_id: Some(id),
            service_port: self.service_port,
            service_ports: self.service_ports,
            rampdown: self.rampdown,
            id_set: PhantomData {},
            port_set: PhantomData {},
            ports_set: PhantomData {},
        }
    }
    #[must_use]
    pub fn with_service_port(self, port: u16) -> ChartBuilder<IdSet, Yes, No> {
        ChartBuilder {
            header: self.header,
            discovery_port: self.discovery_port,
            service_id: self.service_id,
            service_port: Some(port),
            service_ports: Vec::new(),
            rampdown: self.rampdown,
            id_set: PhantomData {},
            port_set: PhantomData {},
            ports_set: PhantomData {},
        }
    }
    #[must_use]
    pub fn with_service_ports(self, ports: Vec<u16>) -> ChartBuilder<IdSet, No, Yes> {
        ChartBuilder {
            header: self.header,
            discovery_port: self.discovery_port,
            service_id: self.service_id,
            service_port: None,
            service_ports: ports,
            rampdown: self.rampdown,
            id_set: PhantomData {},
            port_set: PhantomData {},
            ports_set: PhantomData {},
        }
    }
    #[must_use]
    pub fn with_header(mut self, header: u64) -> ChartBuilder<IdSet, PortSet, PortsSet> {
        self.header = header;
        self
    }
    #[must_use]
    pub fn with_discovery_port(mut self, port: u16) -> ChartBuilder<IdSet, PortSet, PortsSet> {
        self.discovery_port = port;
        self
    }
    /// # Panics
    /// panics if min is larger then max
    #[must_use]
    pub fn with_rampdown(
        mut self,
        min: Duration,
        max: Duration,
        rampdown: Duration,
    ) -> ChartBuilder<IdSet, PortSet, PortsSet> {
        assert!(
            min <= max,
            "minimum duration: {min:?} must be smaller or equal to the maximum: {max:?}"
        );
        self.rampdown = interval::Params { rampdown, min, max };
        self
    }
}

impl ChartBuilder<Yes, No, No> {
    /// # Errors
    /// If a discovery port was set this errors if it could not be opened. If no port was
    /// set this errors if no port on the system could be opened.
    pub fn custom_msg<Msg>(self, msg: Msg) -> Result<Chart<Msg>, Error>
    where
        Msg: Debug + Serialize + Clone,
    {
        let sock = open_socket(self.discovery_port)?;
        Ok(Chart {
            header: self.header,
            service_id: self.service_id.unwrap(),
            msg,
            sock: Arc::new(sock),
            map: Arc::new(dashmap::DashMap::new()),
            interval: self.rampdown.into(),
        })
    }
}

impl ChartBuilder<Yes, Yes, No> {
    /// # Errors
    /// If a discovery port was set this errors if it could not be opened. If no port was
    /// set this errors if no port on the system could be opened.
    pub fn finish(self) -> Result<Chart<Port>, Error> {
        let sock = open_socket(self.discovery_port)?;
        Ok(Chart {
            header: self.header,
            service_id: self.service_id.unwrap(),
            msg: Port(self.service_port.unwrap()),
            sock: Arc::new(sock),
            map: Arc::new(dashmap::DashMap::new()),
            interval: self.rampdown.into(),
        })
    }
}

impl ChartBuilder<Yes, No, Yes> {
    /// # Errors
    /// If a discovery port was set this errors if it could not be opened. If no port was
    /// set this errors if no port on the system could be opened.
    pub fn finish(self) -> Result<Chart<Ports>, Error> {
        let sock = open_socket(self.discovery_port)?;
        Ok(Chart {
            header: self.header,
            service_id: self.service_id.unwrap(),
            msg: Ports(self.service_ports),
            sock: Arc::new(sock),
            map: Arc::new(dashmap::DashMap::new()),
            interval: self.rampdown.into(),
        })
    }
}

fn open_socket(port: u16) -> Result<UdpSocket, Error> {
    assert_ne!(port, 0);

    let interface = Ipv4Addr::from([0, 0, 0, 0]);
    let multiaddr = Ipv4Addr::from([224, 0, 0, 251]);

    use socket2::{Domain, SockAddr, Socket, Type};
    use Error::*;
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, None).map_err(Construct)?;
    sock.set_reuse_port(true).map_err(SetReuse)?; // allow binding to a port already in use
    sock.set_broadcast(true).map_err(SetBroadcast)?; // enable udp broadcasting
    sock.set_multicast_loop_v4(true).map_err(SetMulticast)?; // send broadcast to self

    let address = SocketAddr::from((interface, port));
    let address = SockAddr::from(address);
    sock.bind(&address).map_err(Bind)?;
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
            .finish()
            .unwrap();
        let _ = chart.our_service_port();
    }

    #[tokio::test]
    async fn with_service_ports() {
        let chart = ChartBuilder::new()
            .with_id(0)
            .with_service_ports(vec![1, 2])
            .finish()
            .unwrap();
        let _ = chart.our_service_ports();
    }

    #[tokio::test]
    async fn custom_msg() {
        let chart = ChartBuilder::new().with_id(0).custom_msg("hi").unwrap();
        let _ = chart.our_msg();
    }
}
