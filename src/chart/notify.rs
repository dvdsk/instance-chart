use super::{Id, Entry};

use std::fmt::Debug;
use std::net::IpAddr;
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

pub struct Notify<const N: usize, T: Debug + Clone>(pub(super) broadcast::Receiver<(Id, Entry<[T; N]>)>);

impl<T: Debug + Clone> Notify<1, T> {
    pub async fn recv_one(&mut self) -> Result<(Id, IpAddr, T), RecvError> {
        let (id, ip, [msg]) = self.recv().await?;
        Ok((id, ip, msg))
    }
}

impl<const N: usize, T: Debug + Clone> Notify<N, T> {
    pub async fn recv(&mut self) -> Result<(Id, IpAddr, [T; N]), RecvError> {
        let (id, entry) = self.0.recv().await?;
        Ok((id, entry.ip, entry.msg))
    }
}

impl Notify<1, u16> {
    pub async fn recv_addr(&mut self) -> Result<(Id, SocketAddr), RecvError> {
        let (id, ip, [port]) = self.recv().await?;
        Ok((id, SocketAddr::new(ip, port)))
    }
}

impl<const N: usize> Notify<N, u16> {
    pub async fn recv_addresses(&mut self) -> Result<[(Id, SocketAddr); N], RecvError> {
        let (id, ip, ports) = self.recv().await?;
        Ok(ports.map(|p| (id, SocketAddr::new(ip, p))))
    }
}
