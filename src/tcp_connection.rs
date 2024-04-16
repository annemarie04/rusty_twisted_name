use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, sync::{mpsc::{channel, Sender}, Arc}, thread::{self, Builder},
};
use rand::{Rng, thread_rng};

use crate::{packet::DNSPacket, parser::PacketParser, server::{DNSServer, ServerContext}, stub_resolver, writer::PacketWriter};

pub struct TCPServer {
    // context: Arc<ServerContext>,
    senders: Vec<Sender<TcpStream>>,
    thread_count: usize,
}

impl TCPServer {
    pub fn new(thread_count: usize) -> TCPServer {
        TCPServer {
            // context: context,
            senders: Vec::new(),
            thread_count: thread_count,
        }
    }
}

impl DNSServer for TCPServer {
    fn run_server(mut self) {
        println!("Running TCP server ...");
        let socket = TcpListener::bind(("0.0.0.0:2053")).expect("Error binding TCP socket");
        let mut handlers = Vec::<thread::JoinHandle<()>>::new();
        
        // Spawn threads
        for thread_id in 0..self.thread_count {
            let (tx, rx) = channel();
            self.senders.push(tx);

            // let context = self.context.clone();

            let name = "TCPServer-request-".to_string() + &thread_id.to_string();
            let _worker = match Builder::new().name(name).spawn(move || {
                loop {
                    println!("Looping TCP worker thread no {:?}", thread_id);
                    let mut stream = match rx.recv() {
                        Ok(x) => x,
                        Err(_) => continue,
                    };

                    // Parse the received packet
                    let mut packet_parser = PacketParser::new();
                    PacketParser::tcp_stream_to_bytes(&mut packet_parser, &mut stream);
                    let request = DNSPacket::get_dns_packet(&mut packet_parser);
                
                    // Print packet details 
                    DNSPacket::print_packet(&request);
                    panic!("Packet received!");
                    // Get the answer for the current request by forwarding
                    let mut response = stub_resolver::handle_query(request);
                    
                    // Prepare response for sendng
                    let mut response_writer = PacketWriter::new();
                    response.write_dns_packet(&mut response_writer);

                    let len = response_writer.position();
                    let data = response_writer.get_range(0, len);

                    // Send response
                    let _ = stream.write(data);
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                } // End inner thread loop
            }) {
                    Ok(x) => handlers.push(x),
                    Err(e) => println!("Error on joining threads")
                }; // End of Builder
        } // End of threads loop

        let _ = Builder::new().name("TCPServer-receiving".into()).spawn(move || {
            println!("Launching TCP listening thread...");
            for wrap_stream in socket.incoming() {
                let stream = match wrap_stream {
                    Ok(stream) => panic!("oops"), 
                    Err(err) => {
                        println!("Error on receiving from TCP connection.");
                        continue;
                    }
                };

                // Send the TCPStream to a worker to be solved
                let thread_id = thread_rng().gen::<usize>() % self.thread_count;
                match self.senders[self.thread_count].send(stream) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error sending TCP request to worker number {:?}", thread_id);
                    }
                }
            }
        });

        for handle in handlers {
            handle.join().unwrap();
        }
    }
}

// pub fn try_tcp() {
//     let listener = TcpListener::bind("0.0.0.0:53").unwrap();

//     for stream in listener.incoming() {
//         let stream = stream.unwrap();

//         handle_connection(stream);
//     }
// }

// fn handle_connection(mut stream: TcpStream) {
//     let buf_reader = BufReader::new(&mut stream);
//     let http_request: Vec<_> = buf_reader
//         .lines()
//         .map(|result| result.unwrap())
//         .take_while(|line| !line.is_empty())
//         .collect();

//     println!("Request: {:#?}", http_request);

//     let response = "HTTP/1.1 200 OK\r\n\r\n";

//     stream.write_all(response.as_bytes()).unwrap();
// }