use std::io::{self, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::sleep;


use server::DNSServer;
use udp_connection::UDPServer;
use server_config::ServerContext;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::{fs, thread};
use notify::{ RecursiveMode, Watcher, Event};

use crate::{server, server_config, udp_connection};
use crate::tcp_connection::TCPServer;

pub fn init_servers() -> Result<(), Box<dyn std::error::Error>> {

    let old_context: Arc<ServerContext> = Arc::new(ServerContext::new());
    println!("INITIAL CONFIG: {:?}",old_context);
    let (udp_sender, udp_receiver) = mpsc::channel();
    let udp_receiver = Arc::new(Mutex::new(udp_receiver));
    let (tcp_sender, tcp_receiver) = mpsc::channel();
    let tcp_receiver = Arc::new(Mutex::new(tcp_receiver));

    let (tx, rx) = channel();
    let config_path = "config/server_config.json";
    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
        let mut watcher = notify::recommended_watcher(move |res| {
            match res {
                Ok(event) => {
                    // println!("Event found! {:?}", event);
                    let _ = tx.send(event);
                    }
                Err(e) => println!("Error on watcher: {:?}", e),
            }
        })?;
        // Add a path to be watched. All files and directories at this path and
        // below will be monitored for changes.

                println!("Watching for changes...");
                watcher.watch(Path::new(config_path), RecursiveMode::Recursive).expect("Error on watch");

        
        // Set your debounce threshold here
        debounce_events(rx, Duration::from_secs(1), old_context, udp_sender, udp_receiver, tcp_sender, tcp_receiver); 


        // Use a simple loop to keep the application alive as it waits for file events.
        loop {
            std::thread::sleep(std::time::Duration::from_secs(100));
        }
        Ok(())
}
fn debounce_events(rx: Receiver<Event>, debounce_duration: Duration,mut old_context: Arc<ServerContext>, udp_sender: Sender<()>, udp_receiver: Arc<Mutex<Receiver<()>>>, tcp_sender: Sender<()>, tcp_receiver: Arc<Mutex<Receiver<()>>>)  {
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
                    let new_old_context = start_server(old_context.clone(), udp_sender.clone(), udp_receiver.clone(), tcp_sender.clone(), tcp_receiver.clone()).unwrap();
                    old_context = new_old_context;
            }

            // Update the last seen time
            last_seen.insert(path.clone(), now);
        }
    }
}


fn import_config() -> Result<ServerContext, notify::Error> {
    let config_path = "config/server_config.json";
    let config_data = fs::read_to_string(config_path).expect("Unable to read config file");
    let server_context = serde_json::from_str::<ServerContext>(&config_data)
        .map_err(|e| notify::Error::generic(&e.to_string())); // Convert Box<dyn Error> to notify::Error

    server_context
}

fn start_server(old_context: Arc<ServerContext>,udp_sender: Sender<()>, udp_receiver: Arc<Mutex<Receiver<()>>>, tcp_sender: Sender<()>, tcp_receiver: Arc<Mutex<Receiver<()>>>) -> Result<Arc<ServerContext>,notify::Error> {
    println!("Old Context: {:?}", old_context);
    let tcp_server_state = old_context.enable_tcp;
    let udp_server_state = old_context.enable_udp;


    if let Err(e) = import_config() {
        // Wrong config; Keep the current config running
        println!("Failed to import server configuration: {}", e);
        Err(e)
    } else {
        // New config; Make the changes
        let server_context = Arc::new(import_config().unwrap());
        let context_copy = server_context.clone();
        println!("Successfully imported server configuration: {:?}", server_context);

        

        // 
        if old_context != server_context {
            println!("Applying changes... {:?}", udp_server_state);  
            start_udp_server(old_context.clone(), server_context.clone(), udp_server_state, udp_receiver.clone(), udp_sender);    
            start_tcp_server(old_context.clone(), server_context.clone(), tcp_server_state, tcp_receiver.clone(), tcp_sender)
            }
        

        Ok(context_copy)
    }   
} 

fn start_tcp_server(old_context: Arc<ServerContext>, server_context: Arc<ServerContext>, tcp_server_state: bool, receiver: Arc<Mutex<Receiver<()>>>, sender: Sender<()>) {
    

    if server_context.enable_tcp && tcp_server_state == false{
        // If TCP server was down and start is required
        println!("Starting TCP Server..."); 
        let tcp_server = TCPServer::new(Arc::clone(&server_context), receiver);
        thread::spawn(move || {
            tcp_server.run_server();
        });
        
    } else if server_context.enable_tcp && tcp_server_state {
        // IF TCP server is up and restart is required to apply changes
        println!("TCP Server gracefully shutting down...");
        stop_udp_server(old_context, sender);

        println!("Starting TCP Server...");
        let tcp_server = TCPServer::new(Arc::clone(&server_context), receiver);
        thread::spawn(move || {
            tcp_server.run_server();
        });

    } else if !server_context.enable_tcp && tcp_server_state{
        println!("TCP Server gracefully shutting down...");
        stop_tcp_server(old_context.clone(), sender);
        
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
        
    } else if server_context.enable_udp && udp_server_state {
        // IF UDP server is up and restart is required to apply changes
        println!("UDP Server gracefully shutting down...");
        stop_udp_server(old_context, sender);

        println!("Starting UDP Server...");
        let udp_server = UDPServer::new(Arc::clone(&server_context), receiver);
        thread::spawn(move || {
            udp_server.run_server();
        });

    } else if !server_context.enable_udp && udp_server_state{
        println!("UDP Server gracefully shutting down...");
        stop_udp_server(old_context, sender);
    }
}

fn send_tcp_stream(server_context: Arc<ServerContext>) -> io::Result<()> {
    // Connect to the specified address
    let address  = format!("{}:{}", server_context.dns_host, server_context.dns_port);
    let mut stream = TcpStream::connect(address)?;
    let data = "Stop".as_bytes();
    // Send data
    stream.write_all(data)?;

    // Optionally, you can flush the stream to ensure all data is sent immediately
    stream.flush()?;

    println!("Data sent to the server successfully.");

    Ok(())
}
fn stop_udp_server(server_context: Arc<ServerContext>, sender: Sender<()>) {
    for _thread in 0..server_context.thread_count {
        let _ = sender.send(()); // Sending signal via channel as well
    }
    let _ = sender.send(()); // Sending signal via channel as well
    sleep(std::time::Duration::from_secs(2));
    // let _ = sender.send(()); // Sending signal via channel as well
}

fn stop_tcp_server(server_context: Arc<ServerContext>, sender: Sender<()>) {
    for _thread in 0..server_context.thread_count {
        let _ = sender.send(()); // Sending signal via channel as well
    }
    sleep(std::time::Duration::from_secs(2));

    let _ = send_tcp_stream(server_context.clone());
    let _ = sender.send(()); // Sending signal via channel as well
    sleep(std::time::Duration::from_secs(2));
    // let _ = sender.send(()); // Sending signal via channel as well
}