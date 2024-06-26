use futures::future::join_all;
use std::{
    env,
    net::{Ipv4Addr, SocketAddrV4},
};
use tokio::{io::AsyncWriteExt, net::TcpStream, runtime::Builder, task};

type Result<T> = std::result::Result<T, Error>;
type Error = Box<dyn std::error::Error + Send + Sync>;

async fn scan_port(socket: SocketAddrV4) -> Result<u16> {
    let mut stream = TcpStream::connect(socket).await?;
    stream.shutdown().await?;

    Ok(socket.port())
}

fn main() -> Result<()> {
    let ip_addr = env::args().nth(1).expect("no ip address given");
    let ipv4 = ip_addr.parse::<Ipv4Addr>()?;

    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let mut tasks = vec![];
        for p in 1..=65535 {
            let socket = SocketAddrV4::new(ipv4, p);
            tasks.push(task::spawn(scan_port(socket)));
        }

        let results = join_all(tasks).await;

        let open_ports: Vec<u16> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .filter_map(|o| o.ok())
            .collect();

        for port in open_ports {
            println!("Open port {port}");
        }
    });

    Ok(())
}
