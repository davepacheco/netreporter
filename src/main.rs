//
// netreporter: primitive network monitoring tool
//
// This program currently just sends a single ICMP echo ("ping") packet to
// 127.0.0.1.
//

extern crate pnet;

use std::net::IpAddr;

use pnet::packet::Packet;
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::icmp::echo_reply::IcmpCodes;
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv4;
use pnet::transport::TransportReceiver;
use pnet::transport::TransportSender;
use pnet::transport::icmp_packet_iter;

// Name of the program itself
const NR_ARG0 : &'static str = "netreporter";

//
// Buffer size used for incoming ICMP packets.  We set this to 16K as a power of
// two large enough to store typical jumbograms.  (We don't use these today, but
// it may be useful in the future.)
//
const NR_MAX_INCOMING_PACKET_SIZE : usize = 16384;

fn main()
{
    let target : IpAddr = IpAddr::V4("127.0.0.1".parse().unwrap());

    let mut icmp = match NrIcmpContext::new(NR_MAX_INCOMING_PACKET_SIZE) {
        Ok(icmp) => icmp,
        Err(err) => {
            eprintln!("{}: {}", NR_ARG0, err);
            std::process::exit(1);
        }
    };

    match ping_send(&mut icmp, target) {
        Ok(()) => println!("packet sent to {}", target),
        Err(err) => {
            eprintln!("{}: {}", NR_ARG0, err);
            std::process::exit(1);
        }
    }

    ping_recv_one(&mut icmp);
}

//
// NrIcmpContext encapsulates state related to sending and receiving ICMP
// packets.  This includes channels on which to send and receive individual
// packets, plus any other state (e.g., used for sequence numbers or the like).
//
struct NrIcmpContext {
    #[allow(dead_code)]
    nricmp_bufsize : usize,
    nricmp_tx : TransportSender,
    #[allow(dead_code)]
    nricmp_rx : TransportReceiver,
}

impl NrIcmpContext {
    pub fn new(bufsize : usize)
        -> Result<NrIcmpContext, String>
    {
        let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Icmp));

        let (tx, rx) = match pnet::transport::transport_channel(
            bufsize, protocol) {
            Ok((tx, rx)) => (tx, rx),
            Err(error) => {
                return Err(format!(
                    "failed to allocate ICMP transport channel: {}", error));
            }
        };

        Ok(NrIcmpContext {
            nricmp_bufsize: bufsize,
            nricmp_tx : tx,
            nricmp_rx : rx
        })
    }
}

//
// Sends a single ICMP echo ("ping") packet to the target address.
// XXX target address may as well be a V4 address.
//
fn ping_send(icmp : &mut NrIcmpContext, target: IpAddr)
    -> Result<(), String>
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

    match icmp.nricmp_tx.send_to(packet, target) {
        Ok(_) => Ok(()),
        Err(error) => Err(format!("failed to send ICMP packet: {}", error))
    }
}

//
// Listens for an ICMP receive packet and dumps information about it.
// TODO should have a timeout, if we were to stick with this interface.
//
fn ping_recv_one(icmp : &mut NrIcmpContext)
{
    let mut iter = icmp_packet_iter(&mut icmp.nricmp_rx);
    let (packet, addr) = match iter.next() {
        Ok((packet, addr)) => (packet, addr),
        Err(error) => {
            // XXX shouldn't be printing here
            eprintln!("{}: failed to read reply: {}", NR_ARG0, error);
            return;
        }
    };

    println!("ICMP reply from {}:\n{:?}", addr, packet);
}
