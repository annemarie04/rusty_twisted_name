// use serde::{Serialize, Deserialize};

pub trait DNSServer {
    fn run_server(self);
}
pub enum ResolveType{
    Recursive,
    Forward { host: String, port: u16 },
}

pub struct ServerContext {
    // pub authority,
    pub dns_port: u16,
    pub resolve_strategy: ResolveType,
    pub allow_recursive: bool,
    pub enable_udp: bool,
    pub enable_tcp: bool,
    pub enable_api: bool,
}

impl Default for ServerContext {
    fn default() -> Self {
        ServerContext::new()
    }
}

impl ServerContext {
    pub fn new() -> ServerContext {
        ServerContext {
            // authority: Authority::new(),
            // cache: SynchronizedCache::new(),
            // client: Box::new(DnsNetworkClient::new(34255)),
            dns_port: 53,
            // api_port: 5380,
            resolve_strategy: ResolveType::Recursive,
            allow_recursive: true,
            enable_udp: true,
            enable_tcp: true,
            enable_api: true,
            // statistics: ServerStatistics {
            //     tcp_query_count: AtomicUsize::new(0),
            //     udp_query_count: AtomicUsize::new(0),
            // },
        }
    }
}