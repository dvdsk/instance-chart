use std::fmt::Debug;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::time::sleep;
use tracing::info;

use crate::Chart;
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

/// This drives the chart discovery. You can drop the future but the chart
/// will no longer be updated.
#[tracing::instrument]
pub async fn maintain<'de, const N: usize, T>(chart: Chart<N, T>) 
where
    T: 'static + Debug + Clone + Serialize + DeserializeOwned + Sync + Send
{
    use tokio::task::JoinError;
    let f1 = tokio::spawn(handle_incoming(chart.clone()));
    let f2 = tokio::spawn(broadcast_periodically(chart, Duration::from_secs(10)));
    f1.await.accept_err_with(JoinError::is_cancelled).unwrap();
    f2.await.accept_err_with(JoinError::is_cancelled).unwrap();
}

/// Block until `full_size` nodes have been found.
#[tracing::instrument(skip(chart))]
pub async fn found_everyone<const N:usize, T>(chart: &Chart<N, T>, full_size: u16) 
where
    T: 'static + Debug + Clone + Serialize + DeserializeOwned
{
    assert!(full_size > 2, "minimal cluster size is 3");

    while chart.size() < full_size.into() {
        sleep(Duration::from_millis(100)).await;
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
    assert!(full_size > 2, "minimal cluster size is 3");

    let cluster_majority = (f32::from(full_size) * 0.5).ceil() as usize;
    while chart.size() < cluster_majority {
        sleep(Duration::from_millis(100)).await;
    }
    info!("found majority of cluster, ({} nodes)", chart.size());
}
