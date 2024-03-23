pub struct PacketWriter {
    pub buffer: [u8; 512],
    pub position: usize,
}

impl PacketWriter {
    
    pub fn new() -> PacketWriter {
        PacketWriter {
            buffer: [0; 512],
            position: 0,
        }
    }

    // write 1 bit
    fn write(&mut self, value: u8) {
        self.buffer[self.position] = value;
        self.position += 1;
    }

    // write 1 bit
    fn write_u8(&mut self, value: u8){
        self.write(value);
    }

    // write 2 bits
    fn write_u16(&mut self, value: u16){
        self.write((value >> 8) as u8);
        self.write((value & 0xFF) as u8);

    }

    // write 4 bits
    fn write_u32(&mut self, value: u32) {
        self.write(((value >> 24) & 0xFF) as u8);
        self.write(((value >> 16) & 0xFF) as u8);
        self.write(((value >> 8) & 0xFF) as u8);
        self.write(((value >> 0) & 0xFF) as u8);

    }
}