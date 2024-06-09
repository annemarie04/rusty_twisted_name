use std::{net::Ipv4Addr, sync::Arc};

use crate::{packet::{DNSPacket, QueryType, RCode}, recursive_resolver::recursive_lookup, server_config::{ResolveType, ServerContext}, stub_resolver::lookup};

// Handles an incoming packet
pub fn handle_query(mut request: DNSPacket, server_context: Arc<ServerContext>) -> DNSPacket {
    let mut packet = DNSPacket::new();
    packet.header.id = request.header.id;
    packet.header.recursion_desired = true;
    packet.header.recursion_available = true;
    packet.header.query = true;

    if let Some(question) = request.questions.pop() {
        println!("Received query: {:?}", question);

        if let result = resolve(&question.qname, question.qtype, server_context) {
            packet.questions.push(question.clone());
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
    return packet;
}

pub fn resolve(qname: &str, qtype: QueryType, server_context: Arc<ServerContext>) -> DNSPacket{
    let resolver = server_context.resolve_strategy.clone();
    let rd_flag = server_context.allow_recursive;
    match resolver {
        ResolveType::Recursive => {
            recursive_lookup(qname, qtype, rd_flag)
        },
        ResolveType::Forward { host, port } => {
            println!("Forwarding to {:?}", host);
            let host = host.parse::<Ipv4Addr>().unwrap();
            let server = (host, port);
            lookup(qname, qtype, server, rd_flag)
        }
    }
}