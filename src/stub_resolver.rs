use std::net::UdpSocket;

use crate::{packet::{DNSPacket, DNSQuestion, QueryType, RCode}, parser::PacketParser, writer::PacketWriter};

pub fn lookup(qname: &str, qtype: QueryType) -> DNSPacket{
    // Server to query
    let server = ("8.8.8.8", 53);

    // Set up socket connection to server
    let socket = UdpSocket::bind(("0.0.0.0", 43210)).expect("Couldn't connect to server");

    // Build DNS Query Packet
    let query = build_query(qname, qtype);

    // Send the packet
    socket.send_to(&query.buffer[0..query.position], server).expect("Error on sending packet");

    // Recieve the answer
    let mut response_parser = PacketParser::new();
    socket.recv_from(&mut response_parser.buffer).expect("Error on receiving packet");

    // Build DNSPacket
    let packet = DNSPacket::get_dns_packet(&mut response_parser);
    packet
}
pub fn build_query(qname: &str, qtype: QueryType) -> PacketWriter {
        // Init new DNS Packet
        let mut query_packet = DNSPacket::new();

        // Set the Header
        query_packet.header.id = 6666;
        query_packet.header.qd_count = 1;
        query_packet.header.recursion_desired = true;
        
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

// Handles an incoming packet
pub fn handle_query(socket: &UdpSocket) {
    let mut packet_parser = PacketParser::new();

    let (_, src) = socket.recv_from(&mut packet_parser.buffer).expect("Receiving packet error");

    let mut request = DNSPacket::get_dns_packet(&mut packet_parser);

    let mut packet = DNSPacket::new();
    packet.header.id = request.header.id;
    packet.header.recursion_desired = true;
    packet.header.recursion_available = true;
    packet.header.query = true;

    if let Some(question) = request.questions.pop() {
        println!("Received query: {:?}", question);

        if let result = lookup(&question.qname, question.qtype) {
            packet.questions.push(question);
            packet.header.rcode = result.header.rcode;

            for record in result.answers {
                println!("Answer: {:?}", record);
                packet.answers.push(record);
            }

            for record in result.authorities {
                println!("Authority: {:?}", record);
                packet.authorities.push(record);
            }

            for record in result.resources {
                println!("Resource: {:?}", record);
                packet.resources.push(record);
            }
        } else {
            packet.header.rcode = RCode::SERVFAIL;
        }
    } else {
        // Send FORMERR RCODE if a question is not present
        packet.header.rcode = RCode::FORMERR;
    }

    // Encode the response and send it
    let mut response_writer = PacketWriter::new();
    packet.write_dns_packet(&mut response_writer);

    let len = response_writer.position();
    let data = response_writer.get_range(0, len);

    socket.send_to(data, src).expect("Error sending packet to localhost");
}