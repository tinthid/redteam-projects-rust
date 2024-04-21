use std::io::prelude::*;
use std::net::TcpStream;
use std::env;
use tokio::runtime::Builder;
use futures::future::join_all;

fn scan_port(ip_addr: String, port: i32) -> std::io::Result<()> {
    let mut stream = TcpStream::connect(format!("{ip_addr}:{port}"))?;
    stream.write(&[1])?;
    stream.read(&mut [0; 128])?;
    Ok(())
}

fn main() {

    let ip_addr = env::args().nth(1).expect("no ip address given");

    let runtime = Builder::new_multi_thread()
    .worker_threads(256) 
    .enable_all()     
    .build()
    .unwrap();


    runtime.block_on(async {
        let mut tasks = vec![];
        for p in 1..1024 {
            let ip_addr_clone = ip_addr.clone();
            tasks.push(tokio::spawn(async move {
                match scan_port(ip_addr_clone, p) {
                    Ok(_) => Some(p),
                    _ => None,
                }
            }));
        }


        let results = join_all(tasks).await;
        let open_ports: Vec<i32> = results.into_iter()
        .filter_map(|r| r.unwrap()) 
        .collect();

        for port in open_ports {
            println!("Port {} is open", port);
        }

    });

}

