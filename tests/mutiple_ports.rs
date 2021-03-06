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
async fn local_multiple_ports() {
    setup_tracing();

    let cluster_size: u16 = 5;
    let handles: Vec<_> = (0..cluster_size)
        .map(|id| tokio::spawn(node(id.into(), cluster_size)))
        .collect();

    // in the future use Tokio::JoinSet
    select_all(handles).await.0.unwrap();
}

async fn node(id: u64, cluster_size: u16) {
    let reserv_socket1 = UdpSocket::bind("127.0.0.1:0").unwrap();
    let reserv_socket2 = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port1 = reserv_socket1.local_addr().unwrap().port();
    let port2 = reserv_socket2.local_addr().unwrap().port();
    assert_ne!(port1, 0);
    assert_ne!(port2, 0);

    let chart = ChartBuilder::new()
        .with_id(id)
        .with_service_ports([port1, port2])
        .local_discovery(true)
        .finish()
        .unwrap();
    let maintain = discovery::maintain(chart.clone());
    let _ = tokio::spawn(maintain);

    discovery::found_everyone(&chart, cluster_size).await;
    info!("adresses: {:?}", chart.addr_lists_vec());
    info!(
        "adresses: {:?}",
        chart.nth_addr_vec::<1>()
    );
    info!("discovery complete: {chart:?}");
}
