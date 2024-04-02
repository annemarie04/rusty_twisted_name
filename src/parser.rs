// use std::error::Error;

use std::{fmt::Error, thread::current};

// This is a Packet Parser for UDP Packets of 512bytes 
pub struct PacketParser {
    pub buffer: [u8; 512],
    pub position: usize,
}

impl PacketParser {
    
    // New PacketParser 
    // -> stores a packet of max 512 bytes 
    // -> sets parsing position to 0
    pub fn new() -> PacketParser {
        PacketParser {
            buffer: [0; 512],
            position: 0,
        }
    }

    // Get current parsing position
    // fn position(&self) -> usize {
    //     self.position
    // }

    // // Make a step of size step_size while parsing
    // fn step(&mut self, step_size: usize) {
    //     self.position += step_size;
    // }

    // Change position to a given one
    pub fn jump(&mut self, new_position: usize) {
        self.position = new_position;
    }

    // Read 1 byte and move the position
    pub fn parse_byte(&mut self) -> Result<u8, Error> {

        // Get the coresponding byte content in the buffer
        let parsed_byte = self.buffer[self.position]; 

        // Move position to the next byte
        self.position += 1;

        Ok(parsed_byte)
    }

    // Read 1 byte without moving the position
    fn get_byte(&mut self, given_position: usize) -> Result<u8, Error> {

        // Get the coresponding byte content in the buffer
        let parsed_byte = self.buffer[given_position]; 

        Ok(parsed_byte)
    }

    // Parse a range of bytes
    pub fn parse_byte_range(&mut self, start_position: usize, length: usize) -> Result<String, Error> {
        // Check if the range overflows the packet buffer
        if start_position + length >= 512 {
            panic!("Not enough bytes in packet")
        }
        let mut name = "".to_owned();

        for pos in start_position..start_position + length {
            let parsed_byte = self.get_byte(pos).expect("Cannot parse byte.");
            let letter = parsed_byte as char;
            let mut letter_as_string = [0; 2];

            let result = letter.encode_utf8(&mut letter_as_string);
            name.push_str(result);
        }

        Ok(name)
        // let mut ascii_bytes = [0; length];
        // ascii_bytes = self.buffer[start_position..start_position + length as usize];
        // Ok(String::from_utf8_lossy(&ascii_bytes).to_lowercase())
    }

    // Parse 2 bytes; Move position 2 steps
    pub fn parse_u16(&mut self) -> u16 {
        let parsed_bytes = ((self.parse_byte().expect("u16 parse error") as u16) << 8) 
                        | (self.parse_byte().expect("u16 parse error") as u16);

        parsed_bytes
    }

    // Parse 4 bytes; Move position 4 steps
    pub fn parse_u32(&mut self) -> u32 {
        let parsed_bytes = ((self.parse_byte().expect("u32 parse error") as u32) << 24)
                        | ((self.parse_byte().expect("u32 parse error") as u32) << 16)
                        | ((self.parse_byte().expect("u32 parse error") as u32) << 8)
                        | ((self.parse_byte().expect("u32 parse error") as u32) << 0);

        parsed_bytes
    }


    // Read the queried names
    pub fn parse_qname(&mut self) -> String {
        // Query Name Format: ...[length]Label...
        let mut outstr = "".to_owned();
        // // Position variable to parse within the name
        let mut current_position = self.position;
        // print!("Start qname at: {}", self.position);

        // Jumps
        let mut jumped = false;
        let max_jumps = 5;
        let mut jumps_performed = 0;

        // Get a delimiter between labels: "."
        let mut delimiter = ""; 

        loop {
            // Check if the number of jumbs exceeds the maximum
            if jumps_performed > max_jumps {
                panic!("Limit of {} jumps exceeded", max_jumps);
            }
            // print!("Current position: {current_position}");

            // Start reading
            // Get label length first
            let label_length = self.get_byte(current_position).expect("get_byte error");
            // print!("Label Length: {}", label_length);

            // If the most significant bit is set
            // => it is a jump to some other offset in the packet
            if (label_length & 0xC0) == 0xC0 {
                if !jumped {
                    self.jump(self.position + 2);
                }
                // print!("It's a jumping scenario!!!");
            

                let second_byte = self.get_byte(current_position + 1).expect("get_byte error") as u16;
                let offset = (((label_length as u16) ^ 0xC0) << 8) | second_byte;
                current_position = offset as usize;

                // Jump performed
                jumped = true;
                jumps_performed += 1;

                continue;
            }
            else {
                // Move position by one byte to start reading the actual label
                current_position += 1;

                // Last label is empty => length = 0
                // Stop reading at this point
                if label_length == 0 {
                    if !jumped {
                        self.jump(self.position + 1);
                    }
                    // print!("Stops qname at: {}", self.position);
                    return outstr
                }

                // Append delimiter to the output name
                // The first delimiter will be empty
                // The rest of the delimiters will be "."
                outstr.push_str(delimiter);

                // Get the ASCII bytes for the label
                
                let parsed_name = self.parse_byte_range(current_position, (label_length) as usize).expect("parse_byte_range error");
                outstr.push_str(&parsed_name);

                // Modify the delimiter
                delimiter = ".";
                
                current_position += label_length as usize;
            }

        if !jumped {
            self.jump(current_position);
        }
    }
}
}