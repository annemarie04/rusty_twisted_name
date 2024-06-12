pub mod parser;
pub mod packet;
pub mod writer;
pub mod stub_resolver;
pub mod recursive_resolver;
pub mod server;
pub mod tcp_connection;
pub mod udp_connection;
pub mod server_config;
pub mod resolve_strategy;
pub mod cache;
pub mod start_servers;
use cache::Cache;
use packet::{DNSRecord, QueryType, RCode};
use start_servers::init_servers;


fn main() {
    let _ = init_servers();
    // test_cache();
}


// fn launch_tcp_server() {
//     let dns_server = TCPServer::new(5);
//     TCPServer::run_server(dns_server);
// }

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

fn test_cache() {
    let mut cache = Cache::new();

    // Verify that no data is returned when nothing is present
    if cache.lookup("www.google.com", QueryType::A).is_some() {
        panic!()
    }

    // Register a negative cache entry
    cache.store_nxdomain("www.google.com", QueryType::A, 3600);

    // Verify that we get a response, with the NXDOMAIN flag set
    if let Some(packet) = cache.lookup("www.google.com", QueryType::A) {
        assert_eq!(RCode::NXDOMAIN, packet.header.rcode);
    }

    // Register a negative cache entry with no TTL
    cache.store_nxdomain("www.yahoo.com", QueryType::A, 0);

    // And check that no such result is actually returned, since it's expired
    if cache.lookup("www.yahoo.com", QueryType::A).is_some() {
        panic!()
    }

    // Now add some actual records
    let mut records = Vec::new();
    records.push(DNSRecord::A {
        domain: "www.google.com".to_string(),
        addr: "127.0.0.1".parse().unwrap(),
        ttl: 3600000,
    });
    records.push(DNSRecord::A {
        domain: "www.yahoo.com".to_string(),
        addr: "127.0.0.2".parse().unwrap(),
        ttl: 0,
    });
    records.push(DNSRecord::CNAME {
        domain: "www.microsoft.com".to_string(),
        host: "www.somecdn.com".to_string(),
        ttl: 3600000,
    });

    cache.store(&records);

    // Test for successful lookup
    if let Some(packet) = cache.lookup("www.google.com", QueryType::A) {
        // assert_eq!(records[0], packet.answers[0]);
        println!("Cache answer: {:?}", packet.answers[0]);
    } else {
        panic!();
    }

    // Test for failed lookup, since no CNAME's are known for this domain
    if cache.lookup("www.google.com", QueryType::CNAME).is_some() {
        panic!();
    }

    // Check for successful CNAME lookup
    if let Some(packet) = cache.lookup("www.microsoft.com", QueryType::CNAME) {
        assert_eq!(records[2], packet.answers[0]);
    } else {
        panic!();
    }

    // This lookup should fail, since it has expired due to the 0 second TTL
    if cache.lookup("www.yahoo.com", QueryType::A).is_some() {
        panic!();
    }

    let mut records2 = Vec::new();
    records2.push(DNSRecord::A {
        domain: "www.yahoo.com".to_string(),
        addr: "127.0.0.2".parse().unwrap(),
        ttl: 36000000,
    });

    cache.store(&records2);

    // And now it should succeed, since the record has been store
    if !cache.lookup("www.yahoo.com", QueryType::A).is_some() {
        panic!();
    }

    // Check stat counter behavior
    assert_eq!(3, cache.domain_entries.len());
    assert_eq!(
        1,
        cache
            .domain_entries
            .get(&"www.google.com".to_string())
            .unwrap()
            .hits
    );
    assert_eq!(
        2,
        cache
            .domain_entries
            .get(&"www.google.com".to_string())
            .unwrap()
            .updates
    );
    assert_eq!(
        1,
        cache
            .domain_entries
            .get(&"www.yahoo.com".to_string())
            .unwrap()
            .hits
    );
    assert_eq!(
        3,
        cache
            .domain_entries
            .get(&"www.yahoo.com".to_string())
            .unwrap()
            .updates
    );
    assert_eq!(
        1,
        cache
            .domain_entries
            .get(&"www.microsoft.com".to_string())
            .unwrap()
            .updates
    );
    assert_eq!(
        1,
        cache
            .domain_entries
            .get(&"www.microsoft.com".to_string())
            .unwrap()
            .hits
    );
}

