pub struct PacketWriter {
    pub buffer: [u8; 63000],
    pub position: usize,
}

impl PacketWriter {
    
    pub fn new() -> PacketWriter {
        PacketWriter {
            buffer: [0; 63000],
            position: 0,
        }
    }

    // write 1 bit
    fn write(&mut self, value: u8) {
        self.buffer[self.position] = value;
        self.position += 1;
    }

    // write 1 byte
    pub fn write_u8(&mut self, value: u8){
        self.write(value);
    }

    // write 2 bytes
    pub fn write_u16(&mut self, value: u16){
        self.write((value >> 8) as u8);
        self.write((value & 0xFF) as u8);

    }

    // write 4 bytes
    pub fn write_u32(&mut self, value: u32) {
        self.write(((value >> 24) & 0xFF) as u8);
        self.write(((value >> 16) & 0xFF) as u8);
        self.write(((value >> 8) & 0xFF) as u8);
        self.write(((value >> 0) & 0xFF) as u8);

    }

    pub fn write_qname(&mut self, qname: &str){
        for label in qname.split('.') {
            let length = label.len();

            self.write_u8(length as u8);
            for b in label.as_bytes() {
                self.write_u8(*b);
            }
        }

        self.write_u8(0);
    }

    fn set(&mut self, pos: usize, val: u8) {
        self.buffer[pos] = val;
    }

    pub fn set_u16(&mut self, pos: usize, val: u16) {
        self.set(pos, (val >> 8) as u8);
        self.set(pos + 1, (val & 0xFF) as u8);
    }

    // Get current parsing position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get a range of bytes
    pub fn get_range(&mut self, start: usize, len: usize) -> &[u8] {
        &self.buffer[start..start + len as usize]
    }

    //  Methods for hte TCP Server

    pub fn write_label_length(num: u16, array: &mut [u8]) {
        // Check if the array has enough space to accommodate the number
        if array.len() < 2 {
            panic!("Array is too small to write the number: {:?}", array.len());
        }
    
        // Convert the number to two bytes and store them in the array
        array[0] = (num >> 8) as u8;  // Most significant byte
        array[1] = (num & 0xFF) as u8; // Least significant byte
    }

    pub fn concatenate_arrays<T: Copy>(array1: &[T], array2: &[T]) -> Vec<T> {
        let mut concatenated_array = Vec::with_capacity(array1.len() + array2.len());
        concatenated_array.extend_from_slice(array1);
        concatenated_array.extend_from_slice(array2);
        concatenated_array
    }

    pub fn vec_to_array(mut vec: Vec<u8>)-> Result<[u8; 63000], Vec<u8>> {
        let len = vec.len();
        let mut array = [0u8; 63000];
        let mut pos = 0;
        for elem in vec {
            array[pos] = elem;
            pos = pos + 1; // Fill the array with elements from the vector
        }
        
        Ok(array)
    }

}
