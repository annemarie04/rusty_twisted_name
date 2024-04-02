pub mod parser;
pub mod packet;
pub mod writer;
pub mod stub_resolver;
use crate::packet::{DNSPacket, QueryType};
use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:2053").expect("Error binding to localhost");

    loop {
        // Need to use a try here
        stub_resolver::handle_query(&socket);
    }
} 


fn test_query() {
    let qname = "yahoo.com";
    let qtype = QueryType::MX;
    let response = stub_resolver::lookup(qname, qtype);
    print_packet(response);
}

fn print_packet(packet: DNSPacket) {
    println!("{:#?}", packet.header);

        for q in packet.questions {
            println!("{:#?}", q);
        }
        for rec in packet.answers {
            println!("{:#?}", rec);
        }   
        for rec in packet.authorities {
            println!("{:#?}", rec);
        }
        for rec in packet.resources {
            println!("{:#?}", rec);
        }
}



