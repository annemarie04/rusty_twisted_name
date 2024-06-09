use std::net::Ipv4Addr;
use crate::packet::{DNSPacket, QueryType, RCode};
use crate::stub_resolver::lookup;



pub fn recursive_lookup(qname: &str, qtype: QueryType) -> DNSPacket {
    // Set starting root server
    let mut ns = "198.41.0.4".parse::<Ipv4Addr>().unwrap();

    // loop for recursive search
    loop {
        println!("Attempting lookup of {:?} {}. Querying ns {}.", qtype, qname, ns);

        // Launch query
        let ns_copy = ns;
        let server = (ns_copy, 53);
        let response = lookup(qname, qtype, server);



        // Answer reached!
        if !response.answers.is_empty() && response.header.rcode == RCode::NOERROR {
            return response;
        }

        // The name doesn't exist
        if response.header.rcode == RCode::NXDOMAIN {
            return response;
        }

        // Try a new NS based on the records received
        if let Some(new_ns) = response.get_resolved_ns(qname) {
            ns = new_ns;
            continue;
        }

        // Resolve the IP of a NS record
        // If there are no NS records, go with the latest answer
        let new_ns_name = match response.get_unresolved_ns(qname) {
            Some(x) => x,
            None => return response,
        };

        // Start another recursion
        let recursive_response = recursive_lookup(&new_ns_name, QueryType::A);

        // Pick another IP and continue looping
        if let Some(new_ns) = recursive_response.get_random_record() {
            ns = new_ns;
        } else {
            return  response;
        }
    }
}