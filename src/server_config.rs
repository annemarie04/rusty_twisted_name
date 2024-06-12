use serde_derive::{Deserialize, Serialize};

use crate::cache::{Cache, SynchronizedCache};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ResolveType {
    Recursive,
    Forward {
        #[serde(rename = "host")]
        host: String,
        #[serde(rename = "port")]
        port: u16,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerContext {
    pub dns_port: u16,
    pub dns_host: String,
    pub resolve_strategy: ResolveType,
    pub allow_recursive: bool,
    pub enable_udp: bool,
    pub enable_tcp: bool,
    pub thread_count: usize,
    #[serde(skip_serializing, skip_deserializing)]
    pub cache: SynchronizedCache,

}

impl Default for ServerContext {
    fn default() -> Self {
        ServerContext::new()
    }
}

impl ServerContext {
    pub fn new() -> ServerContext {
        ServerContext {
            dns_port: 53,
            dns_host: "0.0.0.0".to_string(),
            resolve_strategy: ResolveType::Recursive,
            allow_recursive: false,
            enable_udp: false,
            enable_tcp: false,
            thread_count: 1,
            cache: SynchronizedCache::new(),
        }
    }
}

impl PartialEq for ServerContext {
    fn eq(&self, other: &Self) -> bool {
        self.dns_host == other.dns_host && self.dns_port == other.dns_port 
        && self.allow_recursive == other.allow_recursive && self.enable_tcp == other.enable_tcp 
        && self.enable_udp == other.enable_udp && self.thread_count == other.thread_count
    }
}

impl Eq for ServerContext {}