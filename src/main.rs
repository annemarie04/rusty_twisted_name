pub mod parser;
pub mod packet;
pub mod writer;
use crate::parser::PacketParser;
use crate::packet::DNSPacket;
    // let mut f = File::open("response_packet.txt")?;

    // println!(parser.buffer);
    // io::stdin().read_line(&mut guess).expect("Failed to read line");
    
    // let guess: u32 = match guess.trim().parse() {
    //     Ok(num) => num,
    //     Err(_) => continue,

    // };
    // Open the text file
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("127.0.0.1:2053")?;

        loop {
        let mut parser = PacketParser::new();
        let (amt, src) = socket.recv_from(&mut parser.buffer)?;
        
        // PRINTING
        // let mut packet_string = "".to_owned();
        // for x in 0..511 {
        //     let x_str = parser.buffer[x].to_string().to_owned();
        //     packet_string.push_str(&x_str);
        // } 
        // println!("{packet_string}");

        let packet = DNSPacket::get_dns_packet(&mut parser);
        println!("{:#?}", packet.header);

        for q in packet.questions {
            println!("{:#?}", q);
        }
        for rec in packet.answers {
            println!("{:#?}", rec);
        }   
        for rec in packet.authorities {
            println!("{:#?}", rec);
        }
        for rec in packet.resources {
            println!("{:#?}", rec);
        }

        }
        

        // socket.send_to(buf, &src)?;
    } // the socket is closed here
    // Ok(())
}

