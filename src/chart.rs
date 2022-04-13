use std::fmt::Debug;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use tracing::debug;

mod interval;
use interval::Interval;
use tracing::trace;

use crate::Id;
mod builder;
pub use builder::ChartBuilder;

use self::builder::Port;
use self::builder::Ports;
use self::interval::Until;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Entry<Msg: Debug + Clone> {
    ip: IpAddr,
    msg: Msg,
}

#[derive(Debug, Clone)]
pub struct Chart<T: Debug + Clone + Serialize> {
    header: u64,
    service_id: Id,
    msg: T,
    sock: Arc<UdpSocket>,
    interval: Interval,
    map: Arc<dashmap::DashMap<Id, Entry<T>>>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct DiscoveryMsg<T> {
    header: u64,
    id: Id,
    msg: T,
}

impl<T: Serialize + Debug + Clone> Chart<T> {
    #[tracing::instrument]
    fn process_buf<'de>(&self, buf: &'de [u8], addr: SocketAddr) -> bool
    where
        T: Serialize + Deserialize<'de> + Debug,
    {
        let DiscoveryMsg::<T> { header, id, msg } = bincode::deserialize(buf).unwrap();
        if header != self.header {
            return false;
        }
        if id == self.service_id {
            return false;
        }
        let old_key = self.map.insert(id, Entry { ip: addr.ip(), msg });
        if old_key.is_none() {
            debug!(
                "added node: id: {id}, address: {addr:?}, n discoverd: ({})",
                self.size()
            );
            true
        } else {
            false
        }
    }
}

impl Chart<Port> {
    #[must_use]
    pub fn our_service_port(&self) -> u16 {
        self.msg.0
    }

    #[must_use]
    pub fn adresses(&self) -> Vec<SocketAddr> {
        self.map
            .iter()
            .map(|m| (m.value().ip, m.value().msg.0))
            .map(SocketAddr::from)
            .collect()
    }
}

impl Chart<Ports> {
    #[must_use]
    pub fn our_service_ports(&self) -> &[u16] {
        &self.msg.0
    }
}

impl<T: Debug + Clone + Serialize> Chart<T> {
    #[must_use]
    pub fn entries(&self) -> Vec<Entry<T>> {
        self.map.iter().map(|m| m.value().clone()).collect()
    }

    /// members discoverd including self
    #[must_use]
    pub fn size(&self) -> usize {
        self.map.len() + 1
    }

    #[must_use]
    pub fn our_id(&self) -> u64 {
        self.service_id
    }

    #[must_use]
    pub fn our_msg(&self) -> &T {
        &self.msg
    }

    #[must_use]
    pub fn discovery_port(&self) -> u16 {
        self.sock.local_addr().unwrap().port()
    }

    #[must_use]
    fn discovery_msg(&self) -> DiscoveryMsg<T> {
        DiscoveryMsg {
            header: self.header,
            id: self.service_id,
            msg: self.msg.clone(),
        }
    }

    #[must_use]
    fn discovery_buf(&self) -> Vec<u8> {
        let msg = self.discovery_msg();
        bincode::serialize(&msg).unwrap()
    }

    #[must_use]
    fn broadcast_soon(&mut self) -> bool {
        let next = self.interval.next();
        next.until() < Duration::from_millis(100)
    }
}

#[tracing::instrument]
pub async fn handle_incoming<T>(mut chart: Chart<T>)
where
    T: Debug + Clone + Serialize + DeserializeOwned,
{
    loop {
        let mut buf = [0; 1024];
        let (_len, addr) = chart.sock.recv_from(&mut buf).await.unwrap();
        trace!("got msg from: {addr:?}");
        let was_uncharted = chart.process_buf(&buf, addr);
        if was_uncharted && !chart.broadcast_soon() {
            chart
                .sock
                .send_to(&chart.discovery_buf(), addr)
                .await
                .unwrap();
        }
    }
}

#[tracing::instrument]
pub async fn broadcast_periodically<T>(mut chart: Chart<T>, period: Duration)
where
    T: Debug + Serialize + Clone,
{
    loop {
        chart.interval.sleep_till_next().await;
        trace!("sending discovery msg");
        broadcast(&chart.sock, &chart.discovery_buf()).await;
    }
}

#[tracing::instrument]
async fn broadcast(sock: &Arc<UdpSocket>, msg: &[u8]) {
    let multiaddr = Ipv4Addr::from([224, 0, 0, 251]);
    let _len = sock.send_to(msg, (multiaddr, 8080)).await.unwrap();
}
