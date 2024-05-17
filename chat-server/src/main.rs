use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::Mutex;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

type Result<T> = std::result::Result<T, Error>;
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;
type Sender<T> = mpsc::UnboundedSender<T>;

#[derive(Debug)]
struct Message {
    from: SocketAddr,
    dst: SocketAddr,
    msg: String,
}

#[derive(Debug)]
struct Peer {
    addr: SocketAddr,
    writer: WriteHalf<TcpStream>,
}

enum Event {
    NewConnection(Peer),
    NewMessage(Message),
}

//TODO: handle disconnect
#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();

    let (broker_tx, broker_rx) = mpsc::unbounded_channel();
    tokio::spawn(broker_handle(broker_rx));

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        let (reader, writer) = tokio::io::split(stream);

        if let Err(e) = handle_connection(addr, broker_tx.clone(), writer) {
            eprintln!("{e}")
        }

        spawn_and_log_error(receive_messages(reader, addr, broker_tx.clone()));
    }
}

fn handle_connection(
    addr: SocketAddr,
    broker_tx: Sender<Event>,
    writer: WriteHalf<TcpStream>,
) -> Result<()> {
    broker_tx.send(Event::NewConnection(Peer { addr, writer }))?;

    broker_tx.send(Event::NewMessage(Message {
        from: "127.0.0.1:8000".parse::<SocketAddr>().unwrap(),
        dst: addr,
        msg: format!("your socket address is: {addr}"),
    }))?;

    Ok(())
}

async fn broker_handle(mut rx: Receiver<Event>) -> Result<()> {
    let mut peers: HashMap<SocketAddr, WriteHalf<TcpStream>> = HashMap::new();
    while let Some(event) = rx.recv().await {
        match event {
            Event::NewConnection(peer) => {
                peers.insert(peer.addr, peer.writer);
            }
            Event::NewMessage(message) => {
                let msg = format!("from {}: {}\n", message.from, message.msg);
                match peers.get_mut(&message.dst) {
                    Some(writer) => {
                        writer.write(msg.as_bytes()).await.unwrap();
                    }
                    None => (),
                }
            }
        }
    }
    Ok(())
}

fn spawn_and_log_error<F>(fut: F) -> tokio::task::JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    tokio::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}", e);
        }
    })
}

async fn receive_messages(
    reader: tokio::io::ReadHalf<TcpStream>,
    addr: SocketAddr,
    broker_tx: Sender<Event>,
) -> Result<()> {
    let mut reader = BufReader::new(reader);
    loop {
        let mut line = String::new();
        let result = reader.read_line(&mut line).await;
        let bytes_read = result?;
        if bytes_read == 0 {
            break;
        }

        let split_result = split_dst_msg(&line);
        if let None = split_result {
            eprintln!("split error");
            continue;
        }
        let (dst, msg) = split_result.unwrap();

        let parse_socket_result = dst.parse::<SocketAddr>();
        if let Err(e) = parse_socket_result {
            eprintln!("{e}");
            continue;
        }
        broker_tx.send(Event::NewMessage(Message {
            from: addr,
            dst: parse_socket_result.unwrap(),
            msg,
        }))?;
    }

    Ok(())
}

fn split_dst_msg(msg: &str) -> Option<(&str, String)> {
    let idx = match msg.find(':') {
        None => return None,
        Some(first_idx) => match msg[first_idx + 1..].find(':') {
            None => return None,
            Some(second_idx) => first_idx + second_idx + 1,
        },
    };

    Some((&msg[..idx], msg[idx + 1..].trim().to_owned()))
}

#[test]
fn test_get_msg() {
    let payload = "127.0.0.1:234: message";
    let (dst, msg) = split_dst_msg(payload).unwrap();

    assert_eq!(dst, "127.0.0.1:234");
    assert_eq!(msg, "message");
}
