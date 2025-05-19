use clap::Parser;
use futures::stream::{self, StreamExt};
use std::time::Duration;
use tokio::net::TcpStream;

#[derive(Parser)]
struct Args {
    host: String,
    #[clap(default_value="1-1024")]
    ports: String,
    #[clap(short, long, default_value="500")]
    timeout_ms: u64,
}

async fn scan_port(host: &str, port: u16, timeout: Duration) -> bool {
    let addr = format!("{}:{}", host, port);
    tokio::time::timeout(timeout, TcpStream::connect(addr))
        .await
        .map(|res| res.is_ok())
        .unwrap_or(false)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let (start, end) = {
        let mut p = args.ports.split('-');
        let s = p.next().unwrap().parse::<u16>().unwrap();
        let e = p.next().unwrap().parse::<u16>().unwrap();
        (s, e)
    };
    let ports: Vec<u16> = (start..=end).collect();
    let timeout = Duration::from_millis(args.timeout_ms);
    let host = args.host.clone();

    let results = stream::iter(ports)
        .map(|port| {
            let host = host.clone();
            tokio::spawn(async move {
                let open = scan_port(&host, port, timeout).await;
                (port, open)
            })
        })
        .buffer_unordered(100)
        .collect::<Vec<_>>()
        .await;

    for res in results {
        if let Ok((port, true)) = res {
            println!("Port {} is open", port);
        }
    }
}
