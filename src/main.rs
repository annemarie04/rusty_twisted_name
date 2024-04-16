pub mod parser;
pub mod packet;
pub mod writer;
pub mod stub_resolver;
pub mod recursive_resolver;
pub mod server;
pub mod tcp_connection;
pub mod udp_connection;
use std::thread;

use server::DNSServer;
use tcp_connection::TCPServer;
use udp_connection::UDPServer;

fn main() {
    launch_tcp_server();  
} 
fn launch_tcp_server() {
    let dns_server = TCPServer::new(5);
    TCPServer::run_server(dns_server);
}

// fn launch_udp_server() {
//     let dns_server = UDPServer::new(5);
//     UDPServer::run_server(dns_server);
// }
// fn test_query() {
//     let qname = "yahoo.com";
//     let qtype = QueryType::MX;
//     let server = ("8.8.8.8".parse::<Ipv4Addr>().unwrap(), 53 as u16);
//     let response = stub_resolver::lookup(qname, qtype, server);
//     print_packet(response);
// }

// fn print_packet(packet: DNSPacket) {
//     println!("{:#?}", packet.header);

//         for q in packet.questions {
//             println!("{:#?}", q);
//         }
//         for rec in packet.answers {
//             println!("{:#?}", rec);
//         }   
//         for rec in packet.authorities {
//             println!("{:#?}", rec);
//         }
//         for rec in packet.resources {
//             println!("{:#?}", rec);
//         }
// }



