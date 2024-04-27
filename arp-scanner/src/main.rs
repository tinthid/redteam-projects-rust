use ipnetwork::Ipv4Network;
use pnet::{
    datalink::{self, Channel::Ethernet, MacAddr},
    packet::{
        arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket},
        ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
        Packet,
    },
};
use std::env;
use std::net::Ipv4Addr;

fn main() {

    let target_network = env::args().nth(1).expect("No network given");
    let cidr = &target_network;

    let subnet = match cidr.parse::<Ipv4Network>() {
        Ok(network) => network,
        Err(_) => {
            eprintln!("Failed to parse network");
            return;
        }
    };

    for ip in subnet.iter() {
        println!("{}", ip);
    }

}
