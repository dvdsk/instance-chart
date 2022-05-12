use futures::future::select_all;
use instance_chart::{discovery, ChartBuilder};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::net::UdpSocket;

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
async fn test_notify() {
    setup_tracing();

    let cluster_size: u16 = 5;
    let handles: Vec<_> = (0..cluster_size)
        .map(|id| tokio::spawn(node(id.into(), cluster_size)))
        .collect();

    // in the future use Tokio::JoinSet
    select_all(handles).await.0.unwrap();
}

async fn node(id: u64, cluster_size: u16) {
    let reserv_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = reserv_socket.local_addr().unwrap().port();
    assert_ne!(port, 0);

    let chart = ChartBuilder::new()
        .with_id(id)
        .with_service_port(port)
        .local_discovery(true)
        .finish()
        .unwrap();
    let maintain = discovery::maintain(chart.clone());
    let _ = tokio::spawn(maintain);

    if id == 0 {
        let mut new = chart.notify();
        let mut discoverd: HashSet<_> = chart.addr_vec().into_iter().collect();

        while discoverd.len() + 1 < cluster_size as usize {
            let (id, ip, msg) = new.recv().await.unwrap();
            let addr = SocketAddr::new(ip, msg[0]);
            discoverd.insert((id, addr));
        }
    } else {
        discovery::found_everyone(&chart, cluster_size).await;
    }
}

#[tokio::test]
async fn test_notify2() {
    setup_tracing();
    use instance_chart::{discovery, ChartBuilder};

    let full_size = 4u16;
    let _handles: Vec<_> = (1..=full_size)
        .into_iter()
        .map(|id| {
            ChartBuilder::new()
                .with_id(id.into())
                .with_service_port(8042 + id)
                .with_discovery_port(8080)
                .local_discovery(true)
                .finish()
                .unwrap()
        })
        .map(discovery::maintain)
        .map(tokio::spawn)
        .collect();

    let chart = ChartBuilder::new()
        .with_id(1)
        .with_service_port(8042)
        .with_discovery_port(8080)
        .local_discovery(true)
        .finish()
        .unwrap();

    let mut node_discoverd = chart.notify();
    let maintain = discovery::maintain(chart.clone());
    let _ = tokio::spawn(maintain); // maintain task will run forever

    while chart.size() < full_size as usize {
        let new = node_discoverd.recv().await.unwrap();
        println!("discoverd new node: {:?}", new);
    }
}
