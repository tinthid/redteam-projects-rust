use core::num;
use std::net::UdpSocket;
use std::io;

fn parse_dns_query(data: &[u8]) {

    let mut end_position= 13;
    let mut start_position  = 13;
    let mut len_of_bytes = data[12] as usize; 

    while len_of_bytes != 0 {
        end_position = start_position + len_of_bytes;

        let domain = String::from_utf8_lossy(
            &data[start_position..end_position]);
        println!("{}", domain);

        start_position = end_position + 1;
        len_of_bytes = data[end_position] as usize;
    }
}

fn main() -> io::Result<()> {

    let socket = UdpSocket::bind("0.0.0.0:53")?;
    println!("UDP server listening on port 53");
    let mut buf = [0; 1024]; 

    loop {
        let (number_of_bytes, src) = socket.recv_from(&mut buf)?;
        parse_dns_query(&buf);
    }

}
