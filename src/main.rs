pub mod parser;
pub mod packet;
pub mod writer;
pub mod stub_resolver;
pub mod recursive_resolver;
pub mod server;
pub mod tcp_connection;
pub mod udp_connection;
pub mod server_config;
use std::path::PathBuf;
use std::{error::Error, path::Path, thread};

use server::DNSServer;
use tcp_connection::TCPServer;
use udp_connection::UDPServer;
use server_config::ServerContext;

use std::sync::mpsc::{channel, Receiver};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::fs;
use notify::{recommended_watcher, RecursiveMode, Watcher, Config, Event, EventKind};

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let context = import_config().unwrap();
    println!("{:?}",context);
    let (tx, rx) = channel();
    let config_path = "../config/server_config.json";
    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
        let mut watcher = notify::recommended_watcher(move |res| {
            match res {
                Ok(event) => {
                    // println!("Event found! {:?}", event);
                    tx.send(event);
                    }
                Err(e) => println!("Error on watcher: {:?}", e),
            }
        })?;
        // Add a path to be watched. All files and directories at this path and
        // below will be monitored for changes.

                println!("Watching for changes...");
                watcher.watch(Path::new(config_path), RecursiveMode::Recursive).expect("Error on watch");

        
        // Print a message to indicate that watching has started
        debounce_events(rx, Duration::from_secs(1)); // Set your debounce threshold here


        // Use a simple loop to keep the application alive as it waits for file events.
        loop {
            std::thread::sleep(std::time::Duration::from_secs(100));
        }
        Ok(())
}

fn debounce_events(rx: Receiver<Event>, debounce_duration: Duration) {
    let mut last_seen: HashMap<PathBuf, Instant> = HashMap::new();

    while let Ok(event) = rx.recv() {
        // println!("Event found! {:?}", event);
        if let Some(path) = event.paths.first() {
            let now = Instant::now();

            if let Some(last_time) = last_seen.get(path) {
                if now.duration_since(*last_time) < debounce_duration {
                    continue; // Skip this event, as it's within the debounce threshold
                }
            }

            // Handle the event, as it's either the first or sufficiently spaced from the last
            if event.kind.is_modify() {
                println!("Config file modified: {:?}", path);
                start_server();
            }

            // Update the last seen time
            last_seen.insert(path.clone(), now);
        }
    }
}


fn import_config() -> Result<ServerContext, notify::Error> {
    let config_path = "../config/server_config.json";
    let config_data = fs::read_to_string(config_path).expect("Unable to read config file");
    let server_context = serde_json::from_str::<ServerContext>(&config_data)
        .map_err(|e| notify::Error::generic(&e.to_string())); // Convert Box<dyn Error> to notify::Error

    server_context
}

fn start_server() {
    let mut udp_server_state = false;
    let mut tcp_server_state = false;
    if let Err(e) = import_config() {
                        // Wrong config; Keep the current config running
                        println!("Failed to import server configuration: {}", e);
                    } else {
                        // New config; Make the changes
                        let server_context = import_config().unwrap();
                        let tcp_server = TCPServer::new(server_context.clone());

                        println!("Successfully imported server configuration: {:?}", server_context);
                        let udp_server = UDPServer::new(server_context.clone());


                        if server_context.enable_udp && udp_server_state == false{
                            let udp_server_handle = thread::spawn(move || {
                                udp_server.run_server();
                                udp_server_state = true;
                            });
                        }
                        if server_context.enable_udp == false && udp_server_state{
                            // drop(udp_server);
                        }
                        if server_context.enable_tcp && tcp_server_state == false{
                            let tcp_server_handle = thread::spawn(move || {
                                tcp_server.run_server();
                                tcp_server_state = true;
                            });
                        }
                }   
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



