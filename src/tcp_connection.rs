use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, sync::{mpsc::{channel, Sender}, Arc}, thread::Builder,
};

use crate::server::{DNSServer, ServerContext};

pub struct TCPServer {
    context: Arc<ServerContext>,
    senders: Vec<Sender<TcpStream>>,
    thread_count: usize,
}

impl TCPServer {
    pub fn new(context: Arc<ServerContext>, thread_count: usize) -> TCPServer {
        TCPServer {
            context: context,
            senders: Vec::new(),
            thread_count: thread_count,
        }
    }
}

impl DNSServer for TCPServer {
    fn run_server(mut self) {
        println!("Running TCP server ...");
        let socket = TcpListener::bind(("0.0.0.0", self.context.dns_port)).expect("Error binding TCP socket");
    
        // Spawn threads
        for thread_id in 0..self.thread_count {
            let (tx, rx) = channel();
            self.senders.push(tx);

            let context = self.context.clone();

            let name = "TCPServer-request-".to_string() + &thread_id.to_string();
            let _ = Builder::new().name(name).spawn(move || {
                loop {
                    let mut stream = match rx.recv() {
                        Ok(x) => x,
                        Err(_) => continue,
                    };
                }

            });

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