use std::net::{Ipv4Addr, UdpSocket};
use crate::recursive_resolver::recursive_lookup;
use crate::{packet::{DNSPacket, DNSQuestion, QueryType, RCode}, parser::PacketParser, writer::PacketWriter};
 
static id: u16 = 6666;
pub fn lookup(qname: &str, qtype: QueryType, server:(Ipv4Addr, u16), rd_flag:bool) -> DNSPacket{

    // Set up socket connection to server
    let socket = UdpSocket::bind(("0.0.0.0", 43210)).expect("Couldn't connect to server");

    // Build DNS Query Packet
    let query = build_query(qname, qtype, rd_flag);

    // Send the packet
    socket.send_to(&query.buffer[0..query.position], server).expect("Error on sending packet");

    // Recieve the answer
    let mut response_parser = PacketParser::new();
    socket.recv_from(&mut response_parser.buffer).expect("Error on receiving packet");

    // Build DNSPacket
    let packet = DNSPacket::get_dns_packet(&mut response_parser);
    packet
}
pub fn build_query(qname: &str, qtype: QueryType, rd_flag: bool) -> PacketWriter {
        // Init new DNS Packet
        let mut query_packet = DNSPacket::new();

        // Set the Header
        query_packet.header.id = id;
        query_packet.header.qd_count = 1;
        query_packet.header.recursion_desired = rd_flag;
        
        // Set the question
        let mut question = DNSQuestion::new();
        question.qname = qname.to_string();
        question.qtype = qtype;
        question.class = 1;
        query_packet.questions.push(question);

        // Write Packet
        let mut req_buffer = PacketWriter::new();
        query_packet.write_dns_packet(&mut req_buffer);

        req_buffer
}
