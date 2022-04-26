use futures::StreamExt;
use indicatif::ProgressBar;
use instance_chart::{discovery, ChartBuilder};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let jobs: Vec<_> = (1024..10_000u16)
        .into_iter()
        .map(port_can_multicast)
        .collect();

    let mut ok = Vec::new();
    let mut bad = Vec::new();
    let pb = ProgressBar::new(jobs.len() as u64);
    let mut jobs = futures::stream::iter(jobs).buffer_unordered(400);
    while let Some(res) = jobs.next().await {
        pb.inc(1);
        match res {
            Ok(port) => {
                ok.push(port)
            }
            Err(port) => bad.push(port),
        }
    }
    pb.finish();
    if ok.len() < bad.len() {
        println!("ports that can be used: {ok:?}");
    } else {
        println!("ports that can not be used: {bad:?}");
    }
}

async fn port_can_multicast(port: u16) -> Result<u16, u16> {
    let a = node(1, 2, port);
    let b = node(2, 2, port);
    let timeout = tokio::time::sleep(Duration::from_millis(1000));

    tokio::select! {
        res = a => res.map_err(|_| port).map(|()| port),
        res = b => res.map_err(|_| port).map(|()| port),
        _ = timeout => Err(port),
    }
}

async fn node(id: u64, cluster_size: u16, port: u16) -> Result<(), ()> {
    let chart = ChartBuilder::new()
        .with_id(id)
        .with_service_port(42)
        .with_discovery_port(port)
        .local_discovery(true)
        .finish()
        .map_err(|_| ())?;
    let maintain = discovery::maintain(chart.clone());
    let discover = discovery::found_everyone(&chart, cluster_size);

    tokio::select! {
        _ = maintain => unreachable!(),
        _ = discover => Ok(()),
    }
}
