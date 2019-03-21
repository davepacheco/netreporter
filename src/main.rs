extern crate pnet;

use std::net::IpAddr;

use pnet::packet::Packet;
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::icmp::echo_reply::IcmpCodes;
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv4;

//
// We make this large enough to support jumbograms.
//
const MAX_INCOMING_PACKET_SIZE : usize = 16384;

fn main()
{
    let target : IpAddr = IpAddr::V4("127.0.0.1".parse().unwrap());
    ping_send(target);
}

fn ping_send(target: IpAddr)
{
    let bufsz = MutableEchoRequestPacket::minimum_packet_size();
    let mut buffer : Vec<u8> = vec![0; bufsz];
    let mut packet = match MutableEchoRequestPacket::new(&mut buffer) {
        Some(p) => p,
        None => {
            //
            // This really should be impossible, since the only way this is
            // documented to fail is when the buffer wasn't large enough.
            //
            panic!("failed to allocate packet");
        }
    };

    packet.set_icmp_type(IcmpTypes::EchoRequest);
    packet.set_icmp_code(IcmpCodes::NoCode);
    packet.set_sequence_number(47);
    packet.set_identifier(48);

    let checksum = {
        let mut packet_bytes = packet.packet();
        let mut packet_for_checksum = pnet::packet::icmp::IcmpPacket::new(
            packet_bytes).unwrap();
        pnet::packet::icmp::checksum(&packet_for_checksum)
    };
    packet.set_checksum(checksum);

    let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Icmp));
    let mut tx = match pnet::transport::transport_channel(
        MAX_INCOMING_PACKET_SIZE, protocol) {
        Ok((tx, _)) => tx,
        Err(error) => {
            println!("failed to allocate ICMP transport channel: {}", error);
            return;
        }
    };

    match tx.send_to(packet, target) {
        Ok(_) => println!("packet sent!"),
        Err(error) => println!("failed to send ICMP packet: {}", error)
    }
}
