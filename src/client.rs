use pnet::datalink::{Channel, NetworkInterface};
use slog::info;

use crate::{
    ExtensionField, NtpExtensionless, ETHERNET_HEADER_LEN, IPV4_HEADER_LEN, LOGGER, UDP_HEADER_LEN,
};

pub fn run_client(interface: &str) {
    let mut pnet_iface: NetworkInterface = NetworkInterface {
        name: "none".to_string(),
        description: "".to_string(),
        index: 0,
        mac: None,
        ips: vec![],
        flags: 0,
    };

    for iface in pnet::datalink::interfaces() {
        if iface.name.eq(interface) {
            pnet_iface = iface;
            break;
        }
    }

    if pnet_iface.name.eq("nonexistent") {
        panic!("could not find interface: {}", interface);
    }

    let (_tx, mut rx) = match pnet::datalink::channel(&pnet_iface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("unknown channel type"),
        Err(e) => panic!("err: {}", e),
    };

    info!(LOGGER, "Listening on {}", pnet_iface.name);

    loop {
        let packet = rx.next().unwrap();
        let eth = pnet::packet::ethernet::EthernetPacket::new(packet).unwrap();
        if eth.get_ethertype() != pnet::packet::ethernet::EtherTypes::Ipv4 {
            continue;
        }
        let udp_packet = match pnet::packet::udp::UdpPacket::new(
            &packet[(ETHERNET_HEADER_LEN + IPV4_HEADER_LEN)..],
        ) {
            Some(pkt) => pkt,
            None => continue,
        };

        if udp_packet.get_source() == 123 {
            let payload = &packet[(ETHERNET_HEADER_LEN + IPV4_HEADER_LEN + UDP_HEADER_LEN)..];
            let extension: ExtensionField = unsafe {
                std::ptr::read(
                    payload[std::mem::size_of::<NtpExtensionless>()..].as_ptr() as *const _
                )
            };
            info!(LOGGER, "value: {}", extension);
        }
    }
}
