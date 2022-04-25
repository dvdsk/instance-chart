use std::fmt::Debug;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::info;

use crate::{Chart, util};
use crate::chart::{handle_incoming, broadcast_periodically};

trait AcceptErr<T, E> {
    fn accept_err_with(self, f: impl FnOnce(&E) -> bool) -> Result<Option<T>, E>;
}

impl<T, E> AcceptErr<T, E> for Result<T, E> {
    fn accept_err_with(self, f: impl FnOnce(&E) -> bool) -> Result<Option<T>, E> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(e) if f(&e) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// This drives the chart discovery. You can drop the future but then the chart
/// will no longer be updated.
#[tracing::instrument]
pub async fn maintain<'de, const N: usize, T>(chart: Chart<N, T>) 
where
    T: 'static + Debug + Clone + Serialize + DeserializeOwned + Sync + Send
{
    use tokio::task::JoinError;
    let f1 = util::spawn(handle_incoming(chart.clone()));
    let f2 = util::spawn(broadcast_periodically(chart, Duration::from_secs(10)));
    f1.await.accept_err_with(JoinError::is_cancelled).unwrap();
    f2.await.accept_err_with(JoinError::is_cancelled).unwrap();
    // TODO these handles do not abort the tasks when the maintain future is dropped
    // make sure they do! (this is causing the too many open files error in the check ports
    // example)
}

/// Block until `full_size` nodes have been found.
#[tracing::instrument(skip(chart))]
pub async fn found_everyone<const N:usize, T>(chart: &Chart<N, T>, full_size: u16) 
where
    T: 'static + Debug + Clone + Serialize + DeserializeOwned
{
    let mut node_discoverd = chart.notify();
    while chart.size() < full_size as usize {
        node_discoverd.recv().await.unwrap();
    }
    info!(
        "found every member of the cluster, ({} nodes)",
        chart.size()
    );
}

/// Block until a majority of nodes have been found. Usefull when implementing vote based
/// consensus such as Raft.
#[tracing::instrument(skip(chart))]
pub async fn found_majority<const N:usize, T>(chart: &Chart<N,T>, full_size: u16) 
where
    T: 'static + Debug + Clone + Serialize + DeserializeOwned
{
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let cluster_majority = (f32::from(full_size) * 0.5).ceil() as usize;

    let mut node_discoverd = chart.notify();
    while chart.size() < cluster_majority {
        node_discoverd.recv().await.unwrap();
    }
    info!("found majority of cluster, ({} nodes)", chart.size());
}
