use ipnetwork::Ipv4Network;
use pnet::{
    datalink::{self, Channel, MacAddr},
    packet::{
        arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket},
        ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
        Packet,
    },
};
use std::env;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};
use std::fs::read_to_string;
use std::collections::HashMap;

fn main() {

    let target_network = env::args().nth(1).expect("No network given");
    let cidr = &target_network;

    let subnet = cidr.parse::<Ipv4Network>().expect("Failed to parse network");
    
    // read manufacture mac address

    let mut mac_address_data = HashMap::new();
    let mac_data = read_to_string("./manuf").unwrap();
    for mac_line in mac_data.lines() {
            let splited_line: Vec<&str> = mac_line.split("\t").map(|e| e.trim()).collect();
            mac_address_data.insert(splited_line[0], splited_line[2]);
    }

    

    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter()
        .find(|iface| iface.ips.iter().any(|ip| ip.network() == subnet.network()))
        .expect("Network interface with specified IP not found");

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("Error creating datalink channel: {}", e),
    };


    let src_ip = interface.ips.iter()
            .find_map(|ip_network| {
                if let ipnetwork::IpNetwork::V4(ip_net) = ip_network {
                    if ip_net.network() == subnet.network() {
                        return Some(ip_net.ip())
                    }
                }
                None
            }).unwrap();
    
    for ip in subnet.iter() {
        send_arp_request(&mut tx, interface.mac.unwrap().clone(), src_ip, ip);
    }


    let timeout = Duration::new(30, 0);
    let start_time = Instant::now();
    while Instant::now().duration_since(start_time) < timeout {
        match rx.next() {
            Ok(packet) => {
                let ethernet_packet = EthernetPacket::new(packet).unwrap();
                let arp_packet = ArpPacket::new(ethernet_packet.payload()).unwrap();
                if arp_packet.get_operation().0 == 2 {

                    let mac_prefix = format!("{:02X}:{:02X}:{:02X}",
                        arp_packet.get_sender_hw_addr().0,
                        arp_packet.get_sender_hw_addr().1,
                        arp_packet.get_sender_hw_addr().2);

                    println!("{:<15} {} {}", 
                        arp_packet.get_sender_proto_addr(),
                        arp_packet.get_sender_hw_addr(),
                        mac_address_data.get(&mac_prefix as &str).unwrap_or(&""));


                }
            },
            Err(e) => {
                eprintln!("An error occurred while reading: {}", e);
                break;
            }
        }
    }

}


fn send_arp_request(
    tx: &mut Box<dyn datalink::DataLinkSender>, 
    src_mac: MacAddr, src_ip: Ipv4Addr, dest_ip: Ipv4Addr) {

        let mut ethernet_buffer = [0u8; 42]; // Ethernet frame header (14 bytes) + ARP packet (28 bytes)
        let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();
    
        ethernet_packet.set_destination(MacAddr::broadcast());
        ethernet_packet.set_source(src_mac);
        ethernet_packet.set_ethertype(EtherTypes::Arp);

        let mut arp_buffer = [0u8; 28];
        let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).unwrap();
        
        arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
        arp_packet.set_protocol_type(EtherTypes::Ipv4);
        arp_packet.set_hw_addr_len(6);
        arp_packet.set_proto_addr_len(4);
        arp_packet.set_operation(ArpOperations::Request);
        arp_packet.set_sender_hw_addr(src_mac);
        arp_packet.set_sender_proto_addr(src_ip);
        arp_packet.set_target_hw_addr(MacAddr::zero());
        arp_packet.set_target_proto_addr(dest_ip);
        
        ethernet_packet.set_payload(arp_packet.packet());
        
        tx.send_to(&ethernet_packet.packet(), None).unwrap().unwrap();
}
