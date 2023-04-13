use futures::future::select_all;
use instance_chart::{discovery, ChartBuilder};
use std::net::UdpSocket;
use tracing::info;

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

#[tokio::test(flavor = "current_thread")]
async fn local_discovery() {
    setup_tracing();

    let cluster_size: u16 = 5;
    let handles: Vec<_> = (0..cluster_size)
        .map(|_| tokio::spawn(node(cluster_size)))
        .collect();

    // in the future use Tokio::JoinSet
    select_all(handles).await.0.unwrap();
}

async fn node(cluster_size: u16) {
    let reserv_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = reserv_socket.local_addr().unwrap().port();
    assert_ne!(port, 0);

    let chart = ChartBuilder::new()
        .with_random_id()
        .with_service_port(port)
        .local_discovery(true)
        .finish()
        .unwrap();
    let maintain = discovery::maintain(chart.clone());
    let _ = tokio::spawn(maintain);
    discovery::found_everyone(&chart, cluster_size).await;

    info!("discovery complete: {chart:?}");
}
