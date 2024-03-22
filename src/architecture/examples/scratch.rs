extern crate architecture;

use architecture::operation;

struct DataSource {
    cursor: usize,
    bytes: Vec<u8>
}

impl DataSource {
    pub fn new() -> Self {
        DataSource {
            cursor: 0,
            bytes: Vec::new()
        }
    }
}

impl operation::ByteStream for DataSource {
    fn get_next(&mut self) -> u8 {
        let result = self.bytes[self.cursor];
        self.cursor += 1;
        result
    }

    fn get_relative(&mut self, position: isize) -> u8 {
        // let index = self.cursor + position;
        // self.bytes[index];
        todo!()
    }

    fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    fn get_current(&mut self) -> u8 {
        self.bytes[self.cursor]
    }

    fn get_at(&mut self, point: usize) -> u8 {
        self.bytes[point]
    }

    fn get_cursor(&mut self) -> usize {
        self.cursor
    }
}

fn main() {
    let reg_set_from_imm_parser = operation::Parser {
        
    };
}