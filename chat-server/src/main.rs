use std::net::SocketAddr;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, Receiver, Sender};

// TODO: Encrypt messages
#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();

    let (tx, _) = broadcast::channel(10);

    loop {
        let (socket, addr) = listener.accept().await.unwrap();

        let tx = tx.clone();
        let rx = tx.subscribe();

        tokio::spawn(handle(socket, addr, tx, rx));
    }
}

async fn handle(
    mut socket: TcpStream,
    addr: SocketAddr,
    tx: Sender<(Vec<u8>, SocketAddr)>,
    mut rx: Receiver<(Vec<u8>, SocketAddr)>,
) {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);

    loop {
        let mut line = String::new();
        tokio::select! {
            // send message
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    break;
                }
                tx.send((line.into_bytes(), addr)).unwrap();
            }
            // receive message
            result = rx.recv() => {
                let (msg, sender_addr) = result.unwrap();
                if addr != sender_addr {
                    writer.write_all(&msg).await.unwrap();
                }
            }
        }
    }
}
