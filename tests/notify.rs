use multicast_discovery::{discovery, ChartBuilder};
use std::net::SocketAddr;
use std::collections::HashSet;
use std::net::UdpSocket;

fn setup_tracing() {
    use tracing_subscriber::{filter, prelude::*};

    let filter = filter::EnvFilter::builder()
        .parse("info,multicast_discovery=debug")
        .unwrap();

    let fmt = tracing_subscriber::fmt::layer().pretty().with_test_writer();

    let _ignore_err = tracing_subscriber::registry()
        .with(filter)
        .with(fmt)
        .try_init();
}

#[tokio::test(flavor = "current_thread")]
async fn test_notify() {
    setup_tracing();

    let cluster_size: u16 = 5;
    let handles: Vec<_> = (0..cluster_size)
        .map(|id| tokio::spawn(node(id.into(), cluster_size)))
        .collect();

    for h in handles {
        h.await.unwrap();
    }
}

async fn node(id: u64, cluster_size: u16) {
    let reserv_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = reserv_socket.local_addr().unwrap().port();
    assert_ne!(port, 0);

    let chart = ChartBuilder::new()
        .with_id(id)
        .with_service_port(port)
        .finish()
        .unwrap();
    let maintain = discovery::maintain(chart.clone());
    let _ = tokio::spawn(maintain);

    if id == 0 {
        let mut new = chart.notify().await;
        let mut discoverd: HashSet<_> = chart.iter_addr().into_iter().collect();

        while discoverd.len() + 1 < cluster_size as usize {
            let (_id, ip, msg) = new.recv().await.unwrap();
            let addr = SocketAddr::new(ip, msg[0]);
            discoverd.insert(addr);
        }
    } else {
        discovery::found_everyone(&chart, cluster_size).await;
    }
}
