use std::net::SocketAddr;

use super::builder::Port;
use super::{Chart, Entry};

impl<const N: usize> Chart<N, Port> {
    #[must_use]
    pub fn adress_arrays(&self) -> Vec<[SocketAddr; N]> {
        self.map
            .iter()
            .map(|m| {
                let Entry { ip, msg: ports } = m.value();
                ports.map(|p| SocketAddr::new(*ip, p))
            })
            .collect()
    }

    #[must_use]
    pub fn adresses_nth<const IDX: usize>(&self) -> Vec<SocketAddr> {
        self.map
            .iter()
            .map(|m| {
                let Entry { ip, msg: ports } = m.value();
                ports.map(|p| SocketAddr::new(*ip, p))[IDX]
            })
            .collect()
    }
}

impl Chart<1, Port> {
    #[must_use]
    pub fn adresses(&self) -> Vec<SocketAddr> {
        self.adresses_nth::<1>()
    }
}
