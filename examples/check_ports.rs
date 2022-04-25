use futures::StreamExt;
use instance_chart::{discovery, ChartBuilder};
use tokio::join;
use std::time::Duration;
use indicatif::ProgressBar;

fn setup_tracing() {
    use tracing_subscriber::{filter, prelude::*};

    let filter = filter::EnvFilter::builder()
        .parse("info,instance_chart=debug")
        .unwrap();

    let fmt = tracing_subscriber::fmt::layer().pretty().with_test_writer();

    let _ignore_err = tracing_subscriber::registry()
        .with(filter)
        .with(fmt)
        .try_init();
}

#[tokio::main]
async fn main() {
    setup_tracing();

    let jobs: Vec<_> = (1024..10_000u16)
        .into_iter()
        .map(port_can_multicast)
        .collect();

    let mut ok = Vec::new();
    let pb = ProgressBar::new(jobs.len() as u64);
    let mut jobs = futures::stream::iter(jobs).buffer_unordered(200);
    while let Some(res) = jobs.next().await {
        pb.inc(1);
        if let Some(port) = res {
            pb.println("found multicast port: {port}");
            ok.push(port)
        }
    }
    pb.finish();
    println!("ports that support multicast: {ok:?}");
}

async fn port_can_multicast(port: u16) -> Option<u16> {
    let a = node(1, 2, port);
    let b = node(2, 2, port);
    let timeout = tokio::time::sleep(Duration::from_millis(500));

    tokio::select! {
        _ = a => {
            Some(port)
        }
        _ = b => {
            Some(port)
        }
        _ = timeout => {
            None
        }
    }
}

async fn node(id: u64, cluster_size: u16, port: u16) {
    let chart = ChartBuilder::new()
        .with_id(id)
        .with_service_port(42)
        .with_discovery_port(port)
        .local_discovery(true)
        .finish()
        .unwrap();
    let maintain = discovery::maintain(chart.clone());
    let discover = discovery::found_everyone(&chart, cluster_size);

    join!(maintain, discover);
}
