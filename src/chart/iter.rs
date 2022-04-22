use std::net::SocketAddr;

use super::builder::Port;
use super::{Chart, Entry, Id};

impl<const N: usize> Chart<N, Port> {
    /// Returns an iterator over each discovered node's socketadresses.
    /// __note: iteration order is random__
    #[must_use]
    pub fn iter_addr_lists<'a>(&'a self) -> IterAddrLists<'a, N> {
        IterAddrLists {
            inner: self.map.iter(),
        }
    }
}

pub struct IterAddrLists<'a, const N: usize> {
    inner: dashmap::iter::Iter<'a, Id, Entry<[u16; N]>>,
}

impl<'a, const N: usize> Iterator for IterAddrLists<'a, N> {
    type Item = [SocketAddr; N];

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.inner.next()?;
        let Entry { ip, msg: ports } = value.value();
        Some(ports.map(|p| SocketAddr::new(*ip, p)))
    }
}

impl<const N: usize> Chart<N, Port> {
    /// Returns an iterator over each discoverd node's nth-socketadress
    /// of each node.
    /// __note: iteration order is random__
    #[must_use]
    pub fn iter_nth_addr<'a, const IDX: usize>(&'a self) -> IterNthAddr<'a, N, IDX> {
        IterNthAddr {
            inner: self.map.iter(),
        }
    }
}

pub struct IterNthAddr<'a, const N: usize, const IDX: usize> {
    inner: dashmap::iter::Iter<'a, Id, Entry<[u16; N]>>,
}

impl<'a, const N: usize, const IDX: usize> Iterator for IterNthAddr<'a, N, IDX> {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.inner.next()?;
        let Entry { ip, msg: ports } = value.value();
        let port = ports[IDX];
        Some(SocketAddr::new(*ip, port))
    }
}

impl<'a> Chart<1, Port> {
    /// Returns an iterator over each discoverd nodes's socketadress
    /// Note iteration order is random
    #[must_use]
    pub fn iter_addr(&'a self) -> IterAddr<'a> {
        IterAddr {
            inner: self.map.iter(),
        }
    }
}

pub struct IterAddr<'a> {
    inner: dashmap::iter::Iter<'a, Id, Entry<[u16; 1]>>,
}

impl<'a> Iterator for IterAddr<'a> {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.inner.next()?;
        let Entry { ip, msg: ports } = value.value();
        let [port] = ports;
        Some(SocketAddr::new(*ip, *port))
    }
}

#[cfg(test)]
mod tests {
    use crate::chart::{Entry, Interval};
    use crate::{Chart, Id};
    use dashmap::DashMap;
    use serde::Serialize;
    use std::collections::HashSet;
    use std::fmt::Debug;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::Arc;
    use tokio::net::UdpSocket;

    impl<const N: usize, T: Serialize + Debug + Clone> Chart<N, T> {
        async fn test<F>(mut gen_kv: F) -> Self
        where
            F: FnMut(u8) -> (Id, Entry<[T; N]>) + Copy,
        {
            let msg = gen_kv(0).1.msg;
            let map: DashMap<Id, Entry<_>> = (1..10).map(gen_kv).collect();
            Self {
                header: 0,
                service_id: 0,
                msg,
                sock: Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap()),
                interval: Interval::test(),
                map: Arc::new(map),
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
        let iter: HashSet<_> = chart.iter_addr().collect();
        let correct: HashSet<_> = (1..10)
            .map(test_kv)
            .map(|(_, e)| e)
            .map(|e| (e.ip, e.msg[0]))
            .map(SocketAddr::from)
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
    async fn iter_n_ports() {
        let chart = Chart::test(entry_3ports).await;
        let iter: HashSet<_> = chart.iter_addr_lists().collect();
        let correct: HashSet<_> = (1..10)
            .map(entry_3ports)
            .map(|(_, e)| e)
            .map(|e| e.msg.map(|p| (e.ip, p)).map(SocketAddr::from))
            .collect();
        assert_eq!(iter, correct)
    }
    #[tokio::test]
    async fn iter_nth_port() {
        let chart = Chart::test(entry_3ports).await;
        let iter: HashSet<_> = chart.iter_nth_addr::<1>().collect();
        let correct: HashSet<_> = (1..10)
            .map(entry_3ports)
            .map(|(_, e)| e)
            .map(|e| (e.ip, e.msg[1]))
            .map(SocketAddr::from)
            .collect();
        assert_eq!(iter, correct)
    }
}
