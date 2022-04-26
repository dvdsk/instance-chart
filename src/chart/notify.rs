use super::{Id, Entry};

use std::fmt::Debug;
use std::net::IpAddr;
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

/// Wait for notifications of new discoveries, buffering up to 16 discoveries, created using
/// [`Chart::notify()`](crate::Chart::notify).
#[derive(Debug)]
pub struct Notify<const N: usize, T: Debug + Clone>(pub(super) broadcast::Receiver<(Id, Entry<[T; N]>)>);

impl<T: Debug + Clone> Notify<1, T> {
    /// await the next discovered instance. Returns the id and custom messag for new node 
    /// when it is discovered. 
    /// # Note
    /// Can only be called on a 
    /// Notify for a chart created with [`ChartBuilder::custom_msg()`](crate::ChartBuilder::custom_msg) 
    /// # Errors
    /// If more the 16 discoveries have been made since this was called this returns
    /// `RecvError::Lagged`
    pub async fn recv_one(&mut self) -> Result<(Id, IpAddr, T), RecvError> {
        let (id, ip, [msg]) = self.recv().await?;
        Ok((id, ip, msg))
    }
}

impl<const N: usize, T: Debug + Clone> Notify<N, T> {
    /// await the next discovered instance. Returns the id and custom messages for new node 
    /// when it is discovered. 
    /// # Note
    /// Can only be called on a 
    /// Notify for a chart created with [`ChartBuilder::custom_msg()`](crate::ChartBuilder::custom_msg) 
    /// # Errors
    /// If more the 16 discoveries have been made since this was called this returns
    /// `RecvError::Lagged`
    pub async fn recv(&mut self) -> Result<(Id, IpAddr, [T; N]), RecvError> {
        let (id, entry) = self.0.recv().await?;
        Ok((id, entry.ip, entry.msg))
    }
}

impl Notify<1, u16> {
    /// await the next discovered instance. Returns the id and service adresses for new node 
    /// when it is discovered. 
    /// # Note
    /// Can only be called on a 
    /// Notify for a chart created with [`ChartBuilder::finish()`](crate::ChartBuilder::finish) 
    /// that had as single service port set.
    /// # Errors
    /// If more the 16 discoveries have been made since this was called this returns
    /// `RecvError::Lagged`
    pub async fn recv_addr(&mut self) -> Result<(Id, SocketAddr), RecvError> {
        let (id, ip, [port]) = self.recv().await?;
        Ok((id, SocketAddr::new(ip, port)))
    }
}

impl<const N: usize> Notify<N, u16> {
    /// await the next discovered instance. Buffers up to 16 discoveries. Returns the id 
    /// and service adresseses for new node when it is discovered. 
    /// # Note 
    /// Can only be called on a 
    /// Notify for a chart created with [`ChartBuilder::finish()`](crate::ChartBuilder::finish) 
    /// that had multiple service ports set.
    /// # Errors
    /// If more the 16 discoveries have been made since this was called this returns
    /// `RecvError::Lagged`
    pub async fn recv_addresses(&mut self) -> Result<(Id, [SocketAddr; N]), RecvError> {
        let (id, ip, ports) = self.recv().await?;
        Ok((id, ports.map(|p| SocketAddr::new(ip, p))))
    }
}
