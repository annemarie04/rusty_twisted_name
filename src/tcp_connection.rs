use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, sync::{mpsc::{self, channel, Sender}, Arc, Mutex}, thread::{self, Builder},
};
use rand::{Rng, thread_rng};

use crate::{packet::DNSPacket, parser::PacketParser, server::DNSServer, stub_resolver, writer::PacketWriter, server_config::ServerContext};

pub struct TCPServer {
    context: Arc<ServerContext>,
    senders: Vec<Sender<TcpStream>>,
    receiver: Arc<Mutex<mpsc::Receiver<()>>>,
}

impl TCPServer {
    pub fn new(server_context: Arc<ServerContext>, receiver: Arc<Mutex<mpsc::Receiver<()>>>) -> TCPServer {
        TCPServer {
            context: server_context,
            senders: Vec::new(),
            receiver: receiver,
        }
    }
}

impl DNSServer for TCPServer {
    fn run_server(mut self) {
        println!("Running TCP server ...");
        let address  = format!("{}:{}", self.context.dns_host, self.context.dns_port);
        let socket = TcpListener::bind(address).expect("Error binding TCP socket");
        let mut handlers = Vec::<thread::JoinHandle<()>>::new();
        let mut is_running = Arc::new(false);
        
        // Spawn threads
        for thread_id in 0..self.context.thread_count {
            let (tx, rx) = channel();
            let receiver = Arc::clone(&self.receiver);
            let mut is_running = is_running.clone();
            self.senders.push(tx);

            // let context = self.context.clone();

            let name = "TCPServer-request-".to_string() + &thread_id.to_string();
            let _worker = match Builder::new().name(name).spawn(move || {
                loop {
                    if let Ok(_) = receiver.lock().unwrap().try_recv() {
                        println!("Shutdown signal received.");
                        println!("Breaking Thread {:?}", thread_id);
                        is_running = Arc::new(true);
                        break;
                    }
                    println!("Looping TCP worker thread no {:?}", thread_id);
                    let mut stream = match rx.try_recv() {
                        Ok(x) => x,
                        Err(_) => continue,
                    };

                    // Parse the received packet
                    let mut packet_parser = PacketParser::new();
                    PacketParser::tcp_stream_to_bytes(&mut packet_parser, &mut stream);
                    let request = DNSPacket::get_dns_packet(&mut packet_parser);
                
                    // Print packet details 
                    DNSPacket::print_packet(&request);
                    // panic!("Packet received!");
                    // Get the answer for the current request by forwarding
                    let mut response = stub_resolver::handle_query(request);
                    
                    // Prepare response for sending
                    let mut response_writer = PacketWriter::new();
                    response.write_dns_packet(&mut response_writer);

                    let length = response_writer.position() as u16;
                    let data = response_writer.get_range(0, length.into());
                    let mut length_label = [0u8; 2];
                    PacketWriter::write_label_length(length, &mut length_label);

                    let vec_data = PacketWriter::concatenate_arrays(&length_label, data);
                    let len = vec_data.len() + 2;

                    let stream_data = PacketWriter::vec_to_array(vec_data).expect("Error converting vector to array.");
                    // Send response
                    let bytes_written = stream.write(&stream_data[0..len as usize]).expect("Error on sending TCP response.");
                    println!("Written {:?} bytes to stream.", bytes_written);
                    // stream.shutdown(std::net::Shutdown::Both).expect("Error on shutting down stream.");
                } // End inner thread loop
            }) {
                    Ok(x) => handlers.push(x),
                    Err(e) => println!("Error on joining threads")
                }; // End of Builder
        } // End of threads loop
        let receiver = Arc::clone(&self.receiver);

        let is_running = is_running.clone();
        let _ = match Builder::new().name("TCPServer-receiving".into()).spawn(move || {
            println!("Launching TCP listening thread...");
            for wrap_stream in socket.incoming() {
                println!("Received something...");
                if *is_running {
                    println!("Shutdown signal received. Shutting down listener...");
                    break;
                }
                let stream = match wrap_stream {
                    Ok(stream) => stream, 
                    Err(err) => {
                        println!("Error on receiving from TCP connection: {:?}.", err);
                        continue;
                    }
                };

                // let bytes_read = stream.read(&mut buffer)?;

                // // Convert the buffer to a UTF-8 string slice
                // if bytes_read > 0 {
                //     let received_data = str::from_utf8(&buffer[..bytes_read])
                //         .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            
                //     // Check if the received data is "Stop"
                //     if received_data.trim() == "Stop" {
                //         println!("Received 'Stop' command.");
                //         break;
                //     } else {
                //         println!("Received data: {}", received_data);
                //     }

                // }
                // Send the TCPStream to a worker to be solved
                let thread_id = thread_rng().gen::<usize>() % (self.context.thread_count - 1);
                match self.senders[thread_id].send(stream) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error sending TCP request to worker number {:?}", thread_id);
                        println!("is_running = {:?}", e);
                        break;
                    }
                }
            }
            })
            {
                Ok(x) => x.join().unwrap(),
                Err(e) => println!("Error on joining threads")
        }; // End of Builder


        for handle in handlers {
            handle.join().unwrap();
        }
        println!("TCP Server is Down.")
    }

    fn shutdown(&self) {
        println!("TCP shutdown");
    }
}