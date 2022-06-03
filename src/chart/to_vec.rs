use crate::Id;
use std::net::SocketAddr;

use super::builder::Port;
use super::{Chart, Entry};

impl<const N: usize> Chart<N, Port> {
    /// Returns an vector with each discovered node's socketadresses.
    /// # Note
    /// - vector order is random
    /// - only availible for Chart configured with
    /// [`ChartBuilder::with_service_ports`](crate::ChartBuilder::with_service_ports)
    /// and build using [`ChartBuilder::finish`](crate::ChartBuilder::finish).
    /// ```rust
    /// # use std::error::Error;
    /// # use instance_chart::{discovery, ChartBuilder};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let chart = ChartBuilder::new()
    ///     .with_id(1)
    /// #   .with_discovery_port(43785)
    ///     .with_service_ports([8042, 8043, 8044])
    ///     .finish()?;
    /// let maintain = discovery::maintain(chart.clone());
    /// let _ = tokio::spawn(maintain); // maintain task will run forever
    /// let port_lists = chart.addr_lists_vec();
    /// #   Ok(())
    /// # }
    /// ```

    // lock poisoning happens only on crash in another thread, in which
    // case panicing here is expected
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn addr_lists_vec(&self) -> Vec<(Id, [SocketAddr; N])> {
        self.map
            .lock()
            .unwrap()
            .iter()
            .map(|(id, entry)| {
                let Entry { ip, msg: ports } = entry;
                let addr = ports.map(|p| SocketAddr::new(*ip, p));
                (*id, addr)
            })
            .collect()
    }
}

impl<const N: usize> Chart<N, Port> {
    /// Returns a vector over each discoverd node's nth-socketadress
    /// # Note
    /// - vector order is random
    /// - only availible for Chart configured with
    /// [`ChartBuilder::with_service_ports`](crate::ChartBuilder::with_service_ports)
    /// and build using [`ChartBuilder::finish`](crate::ChartBuilder::finish).
    ///
    /// # Examples
    /// ```rust
    /// # use std::error::Error;
    /// # use instance_chart::{discovery, ChartBuilder};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let web_server_port = 8043;
    /// let chart = ChartBuilder::new()
    ///     .with_id(1)
    /// #   .with_discovery_port(43784)
    ///     .with_service_ports([8042, web_server_port, 8044])
    ///     .finish()?;
    /// let maintain = discovery::maintain(chart.clone());
    /// let _ = tokio::spawn(maintain); // maintain task will run forever
    /// let web_server_ports = chart.nth_addr_vec::<2>();
    /// #   Ok(())
    /// # }
    /// ```

    // lock poisoning happens only on crash in another thread, in which
    // case panicing here is expected
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn nth_addr_vec<const IDX: usize>(&self) -> Vec<(Id, SocketAddr)> {
        self.map
            .lock()
            .unwrap()
            .iter()
            .map(|(id, entry)| {
                let Entry { ip, msg: ports } = entry;
                let port = ports[IDX];
                (*id, SocketAddr::new(*ip, port))
            })
            .collect()
    }
}

impl<'a> Chart<1, Port> {
    /// Returns a vector over each discoverd nodes's socketadress
    /// # Note
    /// - vector order is random
    /// - only availible for Chart configured with
    /// [`ChartBuilder::with_service_port`](crate::ChartBuilder::with_service_port)
    /// and build using [`ChartBuilder::finish`](crate::ChartBuilder::finish).
    /// ```rust
    /// # use std::error::Error;
    /// # use instance_chart::{discovery, ChartBuilder};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let chart = ChartBuilder::new()
    ///     .with_id(1)
    /// #   .with_discovery_port(43782)
    ///     .with_service_port(8042)
    ///     .finish()?;
    /// let maintain = discovery::maintain(chart.clone());
    /// let _ = tokio::spawn(maintain); // maintain task will run forever
    /// let ports = chart.addr_vec();
    /// #   Ok(())
    /// # }
    /// ```

    // lock poisoning happens only on crash in another thread, in which
    // case panicing here is expected
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn addr_vec(&'a self) -> Vec<(Id, SocketAddr)> {
        self.map
            .lock()
            .unwrap()
            .iter()
            .map(|(id, entry)| {
                let Entry { ip, msg: [port] } = entry;
                (*id, SocketAddr::new(*ip, *port))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::chart::{Entry, Interval};
    use crate::{Chart, Id};
    use serde::Serialize;
    use std::collections::{HashMap, HashSet};
    use std::fmt::Debug;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::{Arc, Mutex};
    use tokio::net::UdpSocket;

    impl<const N: usize, T: Serialize + Debug + Clone> Chart<N, T> {
        pub async fn test<F>(mut gen_kv: F) -> Self
        where
            F: FnMut(u8) -> (Id, Entry<[T; N]>) + Copy,
        {
            let msg = gen_kv(0).1.msg;
            let map: HashMap<Id, Entry<_>> = (1..10).map(gen_kv).collect();
            Self {
                header: 0,
                service_id: 0,
                msg,
                sock: Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap()),
                interval: Interval::test(),
                map: Arc::new(Mutex::new(map)),
                broadcast: tokio::sync::broadcast::channel(1).0,
            }
        }
    }

    #[tokio::test]
    async fn iter_ports() {
        fn test_kv(n: u8) -> (Id, Entry<[u16; 1]>) {
            let ip = IpAddr::V4(Ipv4Addr::new(n, 0, 0, 1));
            let port = 8000 + n as u16;
            (n as u64, Entry { ip, msg: [port] })
        }

        let chart = Chart::test(test_kv).await;
        let iter: HashSet<_> = chart.addr_vec().into_iter().collect();
        let correct: HashSet<_> = (1..10)
            .map(test_kv)
            .map(|(id, e)| (id, SocketAddr::new(e.ip, e.msg[0])))
            .collect();

        assert_eq!(iter, correct)
    }

    fn entry_3ports(n: u8) -> (Id, Entry<[u16; 3]>) {
        let ip = IpAddr::V4(Ipv4Addr::new(n, 0, 0, 1));
        let port1 = 8000 + n as u16;
        let port2 = 7000 + n as u16;
        let port3 = 6000 + n as u16;
        (
            n as u64,
            Entry {
                ip,
                msg: [port1, port2, port3],
            },
        )
    }

    #[tokio::test]
    async fn iter_addr_lists() {
        let chart = Chart::test(entry_3ports).await;
        let iter: HashSet<_> = chart.addr_lists_vec().into_iter().collect();
        let correct: HashSet<_> = (1..10)
            .map(entry_3ports)
            .map(|(id, e)| {
                let addr = e.msg.map(|p| (e.ip, p)).map(SocketAddr::from);
                (id, addr)
            })
            .collect();
        assert_eq!(iter, correct)
    }
    #[tokio::test]
    async fn iter_nth_port() {
        let chart = Chart::test(entry_3ports).await;
        let iter: HashSet<_> = chart.nth_addr_vec::<1>().into_iter().collect();
        let correct: HashSet<_> = (1..10)
            .map(entry_3ports)
            .map(|(id, e)| (id, SocketAddr::new(e.ip, e.msg[1])))
            .collect();
        assert_eq!(iter, correct)
    }
}
