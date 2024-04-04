pub mod parser;
pub mod packet;
pub mod writer;
pub mod stub_resolver;
pub mod recursive_resolver;
use crate::packet::{DNSPacket, QueryType};
use std::net::{Ipv4Addr, UdpSocket};
use std::net::TcpStream;
fn main() {
    if let Ok(stream) = TcpStream::connect("127.0.0.1:8080") {
        println!("Connected to the server!");
    } else {
        println!("Couldn't connect to server...");
    }
} 

fn dns_over_udp() {
    let socket = UdpSocket::bind("0.0.0.0:2053").expect("Error binding to localhost");

    loop {
        // Need to use a try here
        stub_resolver::handle_query(&socket);
    }
}
fn test_query() {
    let qname = "yahoo.com";
    let qtype = QueryType::MX;
    let server = ("8.8.8.8".parse::<Ipv4Addr>().unwrap(), 53 as u16);
    let response = stub_resolver::lookup(qname, qtype, server);
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



