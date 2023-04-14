use instance_chart::{discovery, ChartBuilder};
use std::io::Write;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::task;
use tokio::task::spawn_blocking;

use instance_chart;

async fn print_user(stream: tokio::net::TcpStream, user: String) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();

    print!("\t{user}: {line}");
}

async fn print_incoming(listener: TcpListener) {
    while let Ok((stream, peer_addr)) = listener.accept().await {
        task::spawn(async move {
            print_user(stream, peer_addr.to_string()).await;
        });
    }
}

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

    let listener = TcpListener::bind(("0.0.0.0", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let chart = ChartBuilder::new()
        .with_random_id()
        .with_service_port(port)
        .local_discovery(true)
        .finish()
        .unwrap();

    // run these task forever in the background
    let chart2 = chart.clone();
    tokio::spawn(async move { discovery::maintain(chart2).await });
    tokio::spawn(async move { print_incoming(listener).await });

    spawn_blocking(move || {
        let reader = std::io::stdin();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            for (_, addr) in chart.addr_vec() {
                let mut peer = std::net::TcpStream::connect(addr).unwrap();
                peer.write_all(line.as_bytes()).unwrap();
            }
        }
    })
    .await
    .unwrap();

    unreachable!()
}
