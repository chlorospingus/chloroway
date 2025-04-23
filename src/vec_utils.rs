
pub trait WlMessage {
    /// Write a u32 to self at offset and increment offset by four
    fn write_u32(&mut self, value: &u32, offset: &mut usize);
    /// Write a u16 to self at offset and increment offset by four
    fn write_u16(&mut self, value: &u16, offset: &mut usize);
    /// Write a string to self at offset
    /// and increment offset by string length rounded up to four bytes
    fn write_string(&mut self, str: &String, offset: &mut usize);
    /// Read a u32 from self at offset and increment offset by four
    fn read_u32(&self, offset: &mut usize) -> u32;
    /// Read a u16 from self at offset and increment offset by two
    fn read_u16(&self, offset: &mut usize) -> u16;
    /// Read a string from self at offset 
    /// and increment offset by string length rounded up to four bytes
    fn read_string(&self, offset: &mut usize) -> String;
}

impl WlMessage for Vec<u8> {
    fn write_u32(&mut self, value: &u32, offset: &mut usize) {
        self[*offset..*offset+4].copy_from_slice(&value.to_ne_bytes());
        *offset += 4;
    }

    fn write_u16(&mut self, value: &u16, offset: &mut usize) {
        self[*offset..*offset+2].copy_from_slice(&value.to_ne_bytes());
        *offset += 2;
    }

    fn write_string(&mut self, str: &String, offset: &mut usize) {
        let mut str = str.clone();
        str.push('\0');
        let rounded_len: u32 = (str.len()+3) as u32 & (u32::MAX-3);
        self.write_u32(&rounded_len, offset);
        self[*offset..*offset+str.len()].copy_from_slice(str.as_bytes());
        *offset += rounded_len as usize;
    }

    fn read_u32(&self, offset: &mut usize) -> u32 {
        let res = u32::from_ne_bytes(
            self[*offset..*offset+4]
            .try_into()
            .expect("u32::from_ne_bytes failed in WlEvent::read_u32")
        );
        *offset += 4;
        res
    }

    fn read_u16(&self, offset: &mut usize) -> u16 {
        let res = u16::from_ne_bytes(
            self[*offset..*offset+2]
            .try_into()
            .expect("u32::from_ne_bytes failed in WlEvent::read_u32")
        );
        *offset += 2;
        res
    }

    fn read_string(&self, offset: &mut usize) -> String {
        let str_len = u32::from_ne_bytes(
            self[*offset..*offset+4]
            .try_into()
            .expect("u32::from_ne_bytes failed in WlEvent::read_string")
        );
        *offset += 4;
        let str = String::from_utf8(
            self[*offset..*offset+((str_len-1) as usize)]
            .to_vec()
        ).expect("String::from_utf8 failed in WlEvent::read_string()");
        *offset += (str_len+3 & u32::MAX-3) as usize;
        str
    }
}
