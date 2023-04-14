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

macro_rules! local_discovery {
    ($cluster_size:literal $test_name:ident) => {

        #[tokio::test(flavor = "current_thread")]
        async fn $test_name() {
            setup_tracing();

            let cluster_size: u16 = $cluster_size;
            let handles: Vec<_> = (0..cluster_size)
                .map(|id| tokio::spawn(node(id.into(), cluster_size)))
                .collect();

            // in the future use Tokio::JoinSet
            select_all(handles).await.0.unwrap();
        }
        
    };
}

local_discovery!{2 n_instances_2}
local_discovery!{5 n_instances_5}
local_discovery!{10 n_instances_10}
local_discovery!{50 n_instances_50}

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
    discovery::found_everyone(&chart, cluster_size).await;

    info!("discovery complete: {chart:?}");
}
