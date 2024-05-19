use socket2::{Domain, Protocol, Socket, Type};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn main() {
    let socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)).unwrap();
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    socket.bind(&address.into()).unwrap();

    let mut buf = Vec::with_capacity(1024);
    let mut payloads = vec![];
    loop {
        buf.clear();
        let nbytes = socket.recv(buf.spare_capacity_mut()).unwrap();
        unsafe {
            buf.set_len(nbytes);
        }

        let iph = etherparse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]).unwrap();
        let icmp = etherparse::Icmpv4Slice::from_slice(&buf[iph.slice().len()..nbytes]).unwrap();

        if let etherparse::Icmpv4Type::EchoRequest(icmph) = icmp.icmp_type() {
            let mut payload = [0; 24];
            payload.clone_from_slice(icmp.payload());
            payloads.push((icmph.seq, payload));
            if icmph.seq == icmph.id {
                break;
            }
        }
    }

    payloads.sort_unstable();
    let payloads = payloads
        .iter()
        .flat_map(|payload| payload.1)
        .collect::<Vec<u8>>();

    println!("{:?}", payloads);
}
