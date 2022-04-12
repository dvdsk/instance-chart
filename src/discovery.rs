use std::time::Duration;

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

#[tracing::instrument]
pub async fn maintain(chart: Chart) {
    let f1 = tokio::spawn(handle_incoming(chart.clone()));
    let f2 = tokio::spawn(broadcast_periodically(chart, Duration::from_secs(10)));
    f1.await.accept_err_with(|e| e.is_cancelled()).unwrap();
    f2.await.accept_err_with(|e| e.is_cancelled()).unwrap();
    unreachable!("maintain never returns")
}

#[tracing::instrument]
pub async fn found_everyone(chart: Chart, full_size: u16) {
    assert!(full_size > 2, "minimal cluster size is 3");

    while chart.size() < full_size.into() {
        sleep(Duration::from_millis(100)).await;
    }
    info!(
        "found every member of the cluster, ({} nodes)",
        chart.size()
    );
}

#[tracing::instrument]
pub async fn found_majority(chart: Chart, full_size: u16) {
    assert!(full_size > 2, "minimal cluster size is 3");

    let cluster_majority = (full_size as f32 * 0.5).ceil() as usize;
    while chart.size() < cluster_majority {
        sleep(Duration::from_millis(100)).await;
    }
    info!("found majority of cluster, ({} nodes)", chart.size());
}
