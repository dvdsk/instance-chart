use crate::Id;
use std::net::SocketAddr;

use super::builder::Port;
use super::{Chart, Entry};

impl<const N: usize> Chart<N, Port> {
    /// Get all the `SocketAddr`'s for a given node's `Id`
    ///
    /// # Note
    /// returns None if the node was not in the Chart
    ///
    /// # Performance
    /// This locks the map. if you need adresses for many nodes
    /// is faster to get a vector of them at once [`Self::addr_lists_vec()`]
    /// instead of calling this repeatedly
    ///
    /// # Panics
    /// This function panics when called with the `Id` of the chart instance
    /// it is called on 
    ///
    /// # Examples
    /// ```rust
    /// # use std::error::Error;
    /// # use instance_chart::{discovery, ChartBuilder};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let service_ports = [8042, 8043, 8044];
    /// let chart = ChartBuilder::new()
    ///     .with_id(1)
    /// #   .with_discovery_port(43794)
    ///     .with_service_ports(service_ports)
    ///     .finish()?;
    /// let maintain = discovery::maintain(chart.clone());
    /// let _ = tokio::spawn(maintain); // maintain task will run forever
    /// let from_chart = chart.get_addr_list(2);
    /// assert_eq!(None, from_chart);
    /// #   Ok(())
    /// # }
    /// ```

    // lock poisoning happens only on crash in another thread, in which
    // case panicing here is expected
    #[must_use]
    pub fn get_addr_list(&self, id: Id) -> Option<[SocketAddr; N]> {
        assert_ne!(self.our_id(), id, "Can not call with our own id");
        let map = self.map.lock().unwrap();
        let Entry { ip, msg: ports } = map.get(&id)?;
        let arr = ports.map(|p| SocketAddr::new(*ip, p));
        Some(arr)
    }
}

impl<const N: usize> Chart<N, Port> {
    /// Get a nodes nth `SocketAddr`'s given its `Id`
    ///
    /// # Note
    /// returns None if the node was not in the Chart
    ///
    /// # Panics
    /// This function panics when called with the `Id` of the chart instance
    /// it is called on 
    ///
    /// # Performance
    /// This locks the map. if you need adresses for many nodes
    /// is faster to get a vector of them at once [`Self::addr_lists_vec()`]
    /// instead of calling this repeatedly
    ///
    /// # Examples
    /// ```rust
    /// # use std::error::Error;
    /// # use instance_chart::{discovery, ChartBuilder};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let port = 8043;
    /// let chart = ChartBuilder::new()
    ///     .with_id(1)
    /// #   .with_discovery_port(43794)
    ///     .with_service_ports([8042, port, 8044])
    ///     .finish()?;
    /// let maintain = discovery::maintain(chart.clone());
    /// let _ = tokio::spawn(maintain); // maintain task will run forever
    /// let from_chart = chart.get_nth_addr::<2>(2);
    /// assert_eq!(None, from_chart);
    /// #   Ok(())
    /// # }
    /// ```

    // lock poisoning happens only on crash in another thread, in which
    // case panicing here is expected
    #[must_use]
    pub fn get_nth_addr<const IDX: usize>(&self, id: Id) -> Option<SocketAddr> {
        assert_ne!(self.our_id(), id, "Can not call with our own id");
        let map = self.map.lock().unwrap();
        let Entry { ip, msg: ports } = map.get(&id)?;
        let port = ports[IDX];
        Some(SocketAddr::new(*ip, port))
    }
}

impl<'a> Chart<1, Port> {
    /// Get a nodes `SocketAddr`'s given its `Id`
    ///
    /// # Note
    /// returns None if the node was not in the Chart
    ///
    /// # Panics
    /// This function panics when called with the `Id` of the chart instance
    /// it is called on 
    ///
    /// # Performance
    /// This locks the map. if you need adresses for many nodes
    /// is faster to get a vector of them at once [`Self::addr_lists_vec()`]
    /// instead of calling this repeatedly
    ///
    /// # Examples
    /// ```rust
    /// # use std::error::Error;
    /// # use instance_chart::{discovery, ChartBuilder};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let port = 8043;
    /// let chart = ChartBuilder::new()
    ///     .with_id(1)
    /// #   .with_discovery_port(43794)
    ///     .with_service_ports([port])
    ///     .finish()?;
    /// let maintain = discovery::maintain(chart.clone());
    /// let _ = tokio::spawn(maintain); // maintain task will run forever
    /// let from_chart = chart.get_addr(2);
    /// assert_eq!(None, from_chart);
    /// #   Ok(())
    /// # }
    /// ```

    // lock poisoning happens only on crash in another thread, in which
    // case panicing here is expected
    #[must_use]
    pub fn get_addr(&self, id: Id) -> Option<SocketAddr> {
        assert_ne!(self.our_id(), id, "Can not call with our own id");
        let map = self.map.lock().unwrap();
        let Entry { ip, msg: [port] } = map.get(&id)?;
        Some(SocketAddr::new(*ip, *port))
    }
}

#[cfg(test)]
mod tests {
    use crate::chart::Entry;
    use crate::{Chart, Id};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn get_addr_list() {
        fn test_kv(n: u8) -> (Id, Entry<[u16; 1]>) {
            let ip = IpAddr::V4(Ipv4Addr::new(n, 0, 0, 1));
            let port = 8000 + n as u16;
            (n as u64, Entry { ip, msg: [port] })
        }

        let chart = Chart::test(test_kv).await;
        let iter = chart.get_addr_list(2).unwrap();
        let entry = test_kv(2).1;
        let correct = [SocketAddr::new(entry.ip, entry.msg[0])];

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
    async fn get_nth_addr() {
        let chart = Chart::test(entry_3ports).await;
        let addr = chart.get_nth_addr::<2>(2).unwrap();
        let entry = entry_3ports(2).1;
        let correct = SocketAddr::new(entry.ip, entry.msg[2]);
        assert_eq!(addr, correct)
    }
}
