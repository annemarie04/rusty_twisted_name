pub mod parser;
pub mod packet;
pub mod writer;
pub mod stub_resolver;
pub mod recursive_resolver;
pub mod server;
pub mod tcp_connection;
pub mod udp_connection;
pub mod server_config;
use std::net::Shutdown;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::sleep;


use server::DNSServer;
use udp_connection::UDPServer;
use server_config::ServerContext;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::{clone, fs, thread};
use notify::{ RecursiveMode, Watcher, Event};

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let udp_server_state: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let tcp_server_state: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    
    let old_context: Arc<ServerContext> = Arc::new(ServerContext::new());
    println!("INITIAL CONFIG: {:?}",old_context);
    let (udp_sender, udp_receiver) = mpsc::channel();
    let udp_receiver = Arc::new(Mutex::new(udp_receiver));


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

        
        // Set your debounce threshold here
        debounce_events(rx, Duration::from_secs(1), old_context, udp_sender, udp_receiver); 


        // Use a simple loop to keep the application alive as it waits for file events.
        loop {
            std::thread::sleep(std::time::Duration::from_secs(100));
        }
        Ok(())
}

fn debounce_events(rx: Receiver<Event>, debounce_duration: Duration,mut old_context: Arc<ServerContext>, udp_sender: Sender<()>, udp_receiver: Arc<Mutex<Receiver<()>>>)  {
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
                    let mut new_old_context = start_server(old_context.clone(), udp_sender.clone(), udp_receiver.clone()).unwrap();
                    old_context = new_old_context;
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

fn start_server(old_context: Arc<ServerContext>,udp_sender: Sender<()>, udp_receiver: Arc<Mutex<Receiver<()>>>) -> Result<Arc<ServerContext>,notify::Error> {
    println!("Old Context: {:?}", old_context);
    let mut tcp_server_state = old_context.enable_tcp;
    let mut udp_server_state = old_context.enable_udp;


    if let Err(e) = import_config() {
        // Wrong config; Keep the current config running
        println!("Failed to import server configuration: {}", e);
        Err(e)
    } else {
        // New config; Make the changes
        let server_context = Arc::new(import_config().unwrap());
        let context_copy = server_context.clone();
        println!("Successfully imported server configuration: {:?}", server_context);

        

        // let tcp_server = TCPServer::new(Arc::clone(&server_context));
        if old_context != server_context {
            println!("Applying changes... {:?}", udp_server_state);  
            start_udp_server(old_context, server_context, udp_server_state, udp_receiver.clone(), udp_sender);    
        }
        

        Ok(context_copy)
    }   
} 



fn start_udp_server(old_context: Arc<ServerContext>, server_context: Arc<ServerContext>, udp_server_state: bool, receiver: Arc<Mutex<Receiver<()>>>, sender: Sender<()>) {
    

    if server_context.enable_udp && udp_server_state == false{
        // If UDP server was down and start is required
        println!("Starting UDP Server..."); 
        let udp_server = UDPServer::new(Arc::clone(&server_context), receiver);
        thread::spawn(move || {
            udp_server.run_server();
        });
        // sleep(std::time::Duration::from_secs(20));
        // stop_server(server_context, sender)

        
    } else if server_context.enable_udp && udp_server_state {
        // IF UDP server is up and restart is required to apply changes
        println!("UDP Server gracefully shutting down...");
        stop_server(old_context, sender);

        println!("Starting UDP Server...");
        let udp_server = UDPServer::new(Arc::clone(&server_context), receiver);
        thread::spawn(move || {
            udp_server.run_server();
        });

    } else if !server_context.enable_udp && udp_server_state{
        println!("UDP Server gracefully shutting down...");
        let result = sender.send(());
        println!("Sending.. {:?}", result);
        stop_server(old_context, sender);
    }
}


fn stop_server(server_context: Arc<ServerContext>, sender: Sender<()>) {
    for thread in 0..server_context.thread_count {
        let _ = sender.send(()); // Sending signal via channel as well
    }
    let _ = sender.send(()); // Sending signal via channel as well
    sleep(std::time::Duration::from_secs(2));
    // let _ = sender.send(()); // Sending signal via channel as well
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



