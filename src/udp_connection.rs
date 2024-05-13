use std::{collections::VecDeque, net::{SocketAddr, UdpSocket}, sync::{Arc, Condvar, Mutex}, thread::{self, Builder}};

use crate::{packet::DNSPacket, parser::PacketParser, server::DNSServer, stub_resolver::{self}};
use crate::writer::PacketWriter;
use crate::server_config::ServerContext;
pub struct UDPServer {
    // Using Arc to share ownership between multiple threads
    context: Arc<ServerContext>,
    request_queue: Arc<Mutex<VecDeque<(SocketAddr, DNSPacket)>>>,
    request_cond: Arc<Condvar>,
}

impl UDPServer {
    // pub fn new(context: Arc<ServerContext>, thread_count: usize) -> UDPServer {
    pub fn new(server_context: ServerContext) -> UDPServer {
        UDPServer {
            // context: context,
            request_queue: Arc::new(Mutex::new(VecDeque::new())),
            request_cond: Arc::new(Condvar::new()),
            context: Arc::new(server_context),
        }
    }
}


impl DNSServer for UDPServer {
    fn run_server(self) {
        println!("Running UDP server ...");
        // Bind the UDP socket 
        let address = format!("{}:{}", self.context.dns_host, self.context.dns_port);
        let socket = UdpSocket::bind(address).expect("Error binding UDP socket");
        let mut handlers = Vec::<thread::JoinHandle<()>>::new();

        // Spawn threads for solving queries
        for thread_id in 0..self.context.thread_count {
            let socket_clone = match socket.try_clone() {
                Ok(x) => x,
                Err(e) => {
                    println!("Failed to clone UDP socket");
                    continue;
                }
            };

            // let context = self.context.clone(); // Config Data
            let request_cond = self.request_cond.clone(); // Condition for blocking threads
            let request_queue = self.request_queue.clone(); // queue with requests

            let name = "DNSServer-solving-".to_string() + &thread_id.to_string();
            let builder = thread::Builder::new();
            let _worker = match builder.spawn(move || {
                // let handle = thread::spawn(move || {
                loop {
                    println!("Looping...thread = {:?}", thread_id);
                    // Take request from queue only if lock is aquired
                    let (src, request) = match request_queue.lock().ok()
                                        .and_then(|x| request_cond.wait(x).ok())
                                        .and_then(|mut x| x.pop_front()) {
                                            Some(x) => x,
                                            None => {
                                                println!("Oops...No requests");
                                                continue;
                                            }
                                        };
                    
                    
                    // Print the current request
                    DNSPacket::print_packet(&request);

                    // Get the answer for the current request by forwarding
                    let mut response = stub_resolver::handle_query(request);
                    
                    // Prepare response for sendng
                    let mut response_writer = PacketWriter::new();
                    response.write_dns_packet(&mut response_writer);

                    let len = response_writer.position();
                    let data = response_writer.get_range(0, len);
                    
                    // Send the response
                    let _ = socket_clone.send_to(data, src).expect("Error on sending response");
                } // End of thread loop
            })
            {
                Ok(x) => handlers.push(x),
                Err(e) => println!("Error on joining threads")
            }; // End of Builder
        } // End of threads for

        // Single thread for receiving
        let builder = thread::Builder::new();
        let _receiving_worker = builder.name("UDPServer-receiving".into()).spawn(
            move || {
                loop {
                    println!("Looping on receiving...");
                    // Get packets from UDP socket
                    let mut packet_parser = PacketParser::new();
                    let socket_copy = socket.try_clone().expect("Socket cloning error");
                    let (_, src) = match socket.recv_from(&mut packet_parser.buffer) {
                        Ok(x) => x,
                        Err(e) => {
                            println!("Error on receiving packet on UDP socket: {:?}.", e);
                            continue;
                        }
                    };

                    // Parse the received request
                    let request = DNSPacket::get_dns_packet(&mut packet_parser);
                    
                    // Print received packet
                    DNSPacket::print_packet(&request);
                    // Acquire lock and add request to queue
                    // Workers should be notified using the Condvar
                    match self.request_queue.lock() {
                        Ok(mut queue) => {
                            queue.push_back((src, request)); // Push packet in queue
                            self.request_cond.notify_one(); // Notify one stopped worker
                        }
                        Err(e) => {
                            println!("Error on adding UDP request to processing queue:{:?}", e);
                        }
                    }
                } // End loop
            }); // End Builder and thread

        for handle in handlers {
            handle.join().unwrap();
        }
    }
}