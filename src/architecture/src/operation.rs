pub trait ByteStream {
    // Get the next byte after the previous byte or initial byte.
    fn get_next(&mut self) -> u8;

    // Get a byte relative to the current byte cursor by index.
    fn get_relative(&mut self, position: isize) -> u8;

    fn set_cursor(&mut self, cursor: usize);
    fn get_cursor(&mut self) -> usize;
    fn get_current(&mut self) -> u8;
    fn get_at(&mut self, point: usize) -> u8;
}

pub struct Parser<'a> {
    pub byte_stream: &'a dyn ByteStream,
    pub opcode: u8,
    pub r0_expected: bool,
    pub r1_expected: bool,
    pub imm_expected: bool,
    pub imm_size: u8 // How many bytes the imm should be
}

impl<'a> Parser<'a> {
    
}