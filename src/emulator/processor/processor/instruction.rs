//! Tools for encoding and decoding instructions into and from an intuitive structure.
//!
//! The instruction encoding and decoding format involved an intermediate format. Instructions involve 2 mandatory
//! driver bytes and an optional register byte.
//! - The driver bytes are encoded & decoded through the [Driver] structure which depends on the [Driver0Encoding] &
//! [Driver1Encoding] traits.
//! - The register byte is encoded & decoded through the [Registers] structure which depends on the [RegisterEncoding]
//! trait.
//!
//! Once the instruction data has been decoded into the intermediates, data is conditioned and extracted into a more
//! intuitive instruction structure.
//!
//! Binary instruction format is as follows.
//!
//! | Required | Byte Name | Field               | Size     | Description                                                     |
//! | -------- | --------- | ------------------- | -------- | --------------------------------------------------------------- |
//! | Yes      | Driver 0  | Extension           | 6 bits   | Operation's extension.                                          |
//! | Yes      | Driver 0  | Synchronise         | 1 bits   | Ensure execution is synchronous in respect to other processors. |
//! | Yes      | Driver 0  | Destination Dynamic | 1 bits   | Base the result location off the dynamic operand.               |
//! | Yes      | Driver 1  | Operation           | 4 bits   | Operation to execute.                                           |
//! | Yes      | Driver 1  | Addressing          | 2 bits   | Dynamic operand's addressing method.                            |
//! | Yes      | Driver 1  | Immediate Exponent  | 2 bits   | Immediate input size power on 2.                                |
//! | No       | Register  | Width               | 2 bits   | Operating data size.                                            |
//! | No       | Register  | Static Operand      | 3 bits   | Static register operand.                                        |
//! | No       | Register  | Dynamic Operand     | 3 bits   | Dynamically addressable operand.                                |
//!
//! Immediate 0..8 quantized to 0, 1, 2, 4 and 8.

#![allow(clippy::unusual_byte_groupings)]

pub mod operand;
pub mod operation;

use std::io;
use std::io::Read;
use emulator::processor::processor::instruction::operand::OperandsPresence;
use crate::number;
use super::instruction::operand::{Destination, Dynamic, Operand, Operands, OperandsConstructError};
use super::instruction::operation::{Extension, ExtensionFromCodeInvalid, Operation};
use crate::utility::{Coded, Encodable};

// region: Binary processor bit masks
pub const DRIVER0_EXTENSION_MASK           : u8 = 0b111111_0_0;
pub const DRIVER0_SYNCHRONISE_MASK         : u8 = 0b000000_1_0;
pub const DRIVER0_DYNAMIC_DESTINATION      : u8 = 0b000000_0_1;
pub const DRIVER1_OPERATION_MASK           : u8 = 0b1111_00_00;
pub const DRIVER1_ADDRESSING_MASK          : u8 = 0b0000_11_00;
pub const DRIVER1_ADDRESSING_PARAMETER_MASK: u8 = 0b0000_00_11;
pub const REGISTERS_WIDTH_MASK             : u8 = 0b11_000_000;
pub const REGISTERS_STATIC_OPERAND_MASK    : u8 = 0b00_111_000;
pub const REGISTERS_DYNAMIC_OPERAND_MASK   : u8 = 0b00_000_111;
// endregion

/// Structured data from the driver bytes. All data generated by inherent functions are unchecked. Contains utility
/// functions for coding driver bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Driver {
    /// Operation extension
    pub extension: u8,
    pub operation: u8,
    pub synchronise: bool,
    /// Whether to store the data where the dynamic operand points if its addressing mode supports it.
    pub dynamic_destination: bool,
    /// Addressing mode of the dynamic operand
    pub addressing: u8,
    /// To determine how many bytes the immediate is.
    pub immediate_exponent: u8
}

impl Driver {
    /// Decode the driver bytes into an instance of a driver.
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver;
    ///
    /// let driver = Driver::new([0b001010_0_1, 0b1111_10_01]);
    ///
    /// // Driver 0
    /// assert_eq!(driver.extension, 0b001010);
    /// assert!(!driver.synchronise);
    /// assert!(driver.dynamic_destination);
    ///
    /// // Driver 1
    /// assert_eq!(driver.operation, 0b1111);
    /// assert_eq!(driver.addressing, 0b10);
    /// assert_eq!(driver.immediate_exponent, 0b1);
    /// ```
    pub fn new(bytes: [u8; 2]) -> Self {
        let driver0 = bytes[0];
        let driver1 = bytes[1];

        Driver {
            extension: driver0.extract_extension(),
            operation: driver1.extract_operation(),
            synchronise: driver0.extract_synchronise(),
            dynamic_destination: driver0.extract_dynamic_destination(),
            addressing: driver1.extract_addressing(),
            immediate_exponent: driver1.extract_immediate_exponent(),
        }
    }
}

impl Encodable<[u8; 2]> for Driver {
    /// Encode the current [Driver] instance into a byte tuple which encodes all the driver information and can be
    /// lossless decoded.
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver;
    /// use atln_processor::utility::Encodable;
    ///
    /// let mut driver = Driver {
    ///     operation: 0b1110,
    ///     extension: 0b1010,
    ///     synchronise: true,
    ///     dynamic_destination: false,
    ///     addressing: 0b11,
    ///     immediate_exponent: 0b10
    /// };
    ///
    /// let encoded = driver.encode();
    ///
    /// assert_eq!(encoded[0], 0b001010_1_0);
    /// assert_eq!(encoded[1], 0b1110_11_10);
    /// ```
    fn encode(&mut self) -> [u8; 2] {
        let mut driver0 = 0.set_extension(self.extension);
        driver0 = driver0.set_synchronise(self.synchronise);
        driver0 = driver0.set_dynamic_destination(self.dynamic_destination);

        let mut driver1 = 0.set_operation(self.operation);
        driver1 = driver1.set_addressing(self.addressing);
        driver1 = driver1.set_immediate_exponent(self.immediate_exponent);

        [driver0, driver1]
    }
}

// region: Uint driver traits
pub trait Driver0Encoding {
    fn extract_extension(self) -> u8;

    /// Only the first 6 bits of the extension is used.
    fn set_extension(self, extension: u8) -> u8;
    
    fn extract_synchronise(self) -> bool;
    
    fn set_synchronise(self, lock: bool) -> u8;
    
    fn extract_dynamic_destination(self) -> bool;
    
    fn set_dynamic_destination(self, dynamic_destination: bool) -> u8;
}

impl Driver0Encoding for u8 {
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver0Encoding;
    ///
    /// assert_eq!(0b001101_0_0_u8.extract_extension(), 0b00_001101);
    /// assert_eq!(0b101010_0_1_u8.extract_extension(), 0b00_101010);
    ///```
    fn extract_extension(self) -> u8 {
        (DRIVER0_EXTENSION_MASK & self) >> 2
    }

    /// Only the first 6 bits of the extension is used.
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver0Encoding;
    ///
    /// assert_eq!(0b000000_0_1_u8.set_extension(10), 0b001010_0_1);
    /// assert_eq!(0b101100_0_0_u8.set_extension(0b101100), 0b101100_0_0);
    /// assert_eq!(0b101100_1_0_u8.set_extension(0b101100), 0b101100_1_0);
    ///
    /// // Truncating extension
    /// assert_eq!(0b00000000_0_0_u8.set_extension(0b11_111111), 0b111111_0_0);
    /// assert_eq!(0b00000000_0_1_u8.set_extension(0b11_111110), 0b111110_0_1);
    /// ```
    fn set_extension(self, extension: u8) -> u8 {
        let layer = (0b00_111111 & extension) << 2;
        (!DRIVER0_EXTENSION_MASK & self) | layer
    }

    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver0Encoding;
    ///
    /// assert!(0b000000_1_0_u8.extract_synchronise());
    /// assert!(!0b000000_0_0_u8.extract_synchronise());
    /// assert!(0b001010_1_1_u8.extract_synchronise());
    /// assert!(!0b001010_0_1_u8.extract_synchronise());
    /// ```
    fn extract_synchronise(self) -> bool {
        // Value will always be 1 bit.
        let bit = (DRIVER0_SYNCHRONISE_MASK & self) >> 1;
        bit == 1
    }

    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver0Encoding;
    /// 
    /// assert_eq!(0b000000_0_0_u8.set_synchronise(true), 0b000000_1_0);
    /// assert_eq!(0b000000_1_0_u8.set_synchronise(false), 0b000000_0_0);
    /// assert_eq!(0b000000_0_1_u8.set_synchronise(true), 0b000000_1_1);
    /// assert_eq!(0b111111_0_0_u8.set_synchronise(false), 0b111111_0_0);
    /// ```
    fn set_synchronise(self, lock: bool) -> u8 {
        let layer = (lock as u8) << 1;
        (!DRIVER0_SYNCHRONISE_MASK & self) | layer
    }

    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver0Encoding;
    ///
    /// assert!(0b000000_0_1_u8.extract_dynamic_destination());
    /// assert!(!0b000000_0_0_u8.extract_dynamic_destination());
    /// assert!(0b000000_1_1_u8.extract_dynamic_destination());
    /// assert!(!0b000000_1_0_u8.extract_dynamic_destination());
    /// ```
    fn extract_dynamic_destination(self) -> bool {
        // Value will always be 1 bit.
        (DRIVER0_DYNAMIC_DESTINATION & self) == 1
    }

    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver0Encoding;
    ///
    /// assert_eq!(0b000000_0_0_u8.set_dynamic_destination(true), 0b000000_0_1);
    /// assert_eq!(0b000000_1_0_u8.set_dynamic_destination(true), 0b000000_1_1);
    /// assert_eq!(0b000000_0_1_u8.set_dynamic_destination(false), 0b000000_0_0);
    /// assert_eq!(0b000000_1_1_u8.set_dynamic_destination(false), 0b000000_1_0);
    /// ```
    fn set_dynamic_destination(self, dynamic_destination: bool) -> u8 {
        (!DRIVER0_DYNAMIC_DESTINATION & self) | dynamic_destination as u8
    }
}

pub trait Driver1Encoding {
    fn extract_operation(self) -> u8;

    /// Only the first 4 bits of the operation is used.
    fn set_operation(self, operation: u8) -> u8;
    
    fn extract_addressing(self) -> u8;

    /// Only the first 2 bits of the addressing is used.
    fn set_addressing(self, addressing: u8) -> u8;
    
    fn extract_immediate_exponent(self) -> u8;

    /// Only the first 2 bits of the addressing is used.
    fn set_immediate_exponent(self, immediate_exponent: u8) -> u8;
}

impl Driver1Encoding for u8 {
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver1Encoding;
    ///
    /// assert_eq!(0b1101_00_00_u8.extract_operation(), 0b0000_1101);
    /// assert_eq!(0b1010_01_10_u8.extract_operation(), 0b0000_1010);
    /// ```
    fn extract_operation(self) -> u8 {
        (DRIVER1_OPERATION_MASK & self) >> 4
    }

    /// Only the first 4 bits of the operation is used.
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver1Encoding;
    ///
    /// assert_eq!(0b0001_00_11_u8.set_operation(0b0000_1111), 0b1111_00_11);
    /// assert_eq!(0b1111_00_10_u8.set_operation(0b0000_1001), 0b1001_00_10);
    /// assert_eq!(0b1010_00_10_u8.set_operation(0b0000_1010), 0b1010_00_10);
    ///
    /// // Truncating extension
    /// assert_eq!(0b0000_00_00_u8.set_operation(0b1111_1111), 0b1111_00_00);
    /// assert_eq!(0b0000_10_01_u8.set_operation(0b1111_1111), 0b1111_10_01);
    /// ```
    fn set_operation(self, operation: u8) -> u8 {
        let layer = (0b0000_1111 & operation) << 4;
        (!DRIVER1_OPERATION_MASK & self) | layer
    }

    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver1Encoding;
    ///
    /// assert_eq!(0b0011_10_00_u8.extract_addressing(), 0b000000_10);
    /// assert_eq!(0b1011_11_00_u8.extract_addressing(), 0b000000_11);
    /// assert_eq!(0b0000_00_00_u8.extract_addressing(), 0b000000_00);
    /// ```
    fn extract_addressing(self) -> u8 {
        (DRIVER1_ADDRESSING_MASK & self) >> 2
    }

    /// Only the first 2 bits of the addressing is used.
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver1Encoding;
    ///
    /// assert_eq!(0b0000_11_00_u8.set_addressing(0b000000_00), 0b0000_00_00);
    /// assert_eq!(0b0011_00_00_u8.set_addressing(0b000000_01), 0b0011_01_00);
    /// assert_eq!(0b1011_00_00_u8.set_addressing(0b000000_00), 0b1011_00_00);
    ///
    /// // Truncating extension
    /// assert_eq!(0b0000_00_00_u8.set_addressing(0b111111_11), 0b0000_11_00);
    /// assert_eq!(0b1010_00_01_u8.set_addressing(0b111111_11), 0b1010_11_01);
    /// ```
    fn set_addressing(self, addressing: u8) -> u8 {
        let layer = (0b_000000_11 & addressing) << 2;
        (!DRIVER1_ADDRESSING_MASK & self) | layer
    }

    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver1Encoding;
    ///
    /// assert_eq!(0b0000_00_11_u8.extract_immediate_exponent(), 0b000000_11);
    /// assert_eq!(0b1010_11_01_u8.extract_immediate_exponent(), 0b000000_01);
    /// ```
    fn extract_immediate_exponent(self) -> u8 {
        DRIVER1_ADDRESSING_PARAMETER_MASK & self
    }

    /// Only the first 2 bits of the addressing is used.
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Driver1Encoding;
    ///
    /// assert_eq!(0b0011_00_00_u8.set_immediate_exponent(0b000000_11), 0b0011_00_11);
    /// assert_eq!(0b0000_11_00_u8.set_immediate_exponent(0b000000_10), 0b0000_11_10);
    /// assert_eq!(0b1011_01_00_u8.set_immediate_exponent(0b000000_00), 0b1011_01_00);
    ///
    /// // Truncating extension
    /// assert_eq!(0b0000_00_00_u8.set_immediate_exponent(0b111111_11), 0b0000_00_11);
    /// assert_eq!(0b1011_01_00_u8.set_immediate_exponent(0b111111_10), 0b1011_01_10);
    /// ```
    fn set_immediate_exponent(self, immediate_exponent: u8) -> u8 {
        let layer = 0b000000_11 & immediate_exponent;
        (!DRIVER1_ADDRESSING_PARAMETER_MASK & self) | layer
    }
}
// endregion

/// Register byte encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registers {
    pub width: u8,
    pub x_static: u8,
    pub x_dynamic: u8
}

impl Registers {
    /// Create a new instance from an encoded form of the registers byte.
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Registers;
    /// 
    /// // Test operands, ensure no mirroring occurs.
    /// assert_eq!(Registers::new(0b00__000_001), Registers { width: 0, x_static: 0, x_dynamic: 1 });
    /// assert_eq!(Registers::new(0b11__011_111), Registers { width: 3, x_static: 3, x_dynamic: 7 });
    /// assert_eq!(Registers::new(0b10__000_001), Registers { width: 2, x_static: 0, x_dynamic: 1 });
    /// ```
    pub fn new(encoded: u8) -> Self {
        Self {
            width: encoded.extract_width(),
            x_static: encoded.extract_static(),
            x_dynamic: encoded.extract_dynamic()
        }
    }

    /// Encode this registers data structure into a registers byte which contains the properties of register targeting.
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::Registers;
    ///
    /// assert_eq!(Registers { width: 0, x_static: 0, x_dynamic: 1 }.encode(), 0b00__000_001);
    /// assert_eq!(Registers { width: 3, x_static: 3, x_dynamic: 7 }.encode(), 0b11__011_111);
    /// assert_eq!(Registers { width: 2, x_static: 0, x_dynamic: 1 }.encode(), 0b10__000_001);
    /// ```
    pub fn encode(&self) -> u8 {
        let mut encoded = 0.set_width(self.width);
        encoded = encoded.set_static(self.x_static);
        encoded.set_dynamic(self.x_dynamic)
    }
}

// region: Uint traits
pub trait RegistersEncoding {
    /// Extract the width exponent.
    fn extract_width(self) -> u8;

    /// Set the width exponent.
    fn set_width(self, width: u8) -> u8;

    fn extract_static(self) -> u8;

    fn set_static(self, x_static: u8) -> u8;

    fn extract_dynamic(self) -> u8;

    fn set_dynamic(self, dynamic: u8) -> u8;
}

impl RegistersEncoding for u8 {
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::RegistersEncoding;
    ///
    /// assert_eq!(0b00_000_000.extract_width(), 0b00_000_00);
    /// assert_eq!(0b11_011_110.extract_width(), 0b00_000_11);
    /// assert_eq!(0b10_001_010.extract_width(), 0b00_000_10);
    /// assert_eq!(0b01_000_111.extract_width(), 0b00_000_01);
    /// ```
    fn extract_width(self) -> u8 {
        (REGISTERS_WIDTH_MASK & self) >> 6
    }

    /// Only first 2 bits are used.
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::RegistersEncoding;
    ///
    /// assert_eq!(0b00_000_000.set_width(0b00), 0b00_000_000);
    /// assert_eq!(0b00_011_110.set_width(0b11), 0b11_011_110);
    /// assert_eq!(0b11_001_010.set_width(0b00), 0b00_001_010);
    /// assert_eq!(0b10_000_111.set_width(0b11), 0b11_000_111);
    /// ```
    fn set_width(self, width: u8) -> u8 {
        let layer = (0b000000_11 & width) << 6;
        (!REGISTERS_WIDTH_MASK & self) | layer
    }

    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::RegistersEncoding;
    /// 
    /// assert_eq!(0b10_111_111.extract_static(), 0b00_000_111);
    /// assert_eq!(0b11_101_100.extract_static(), 0b00_000_101);
    /// assert_eq!(0b00_101_010.extract_static(), 0b00_000_101);
    /// assert_eq!(0b01_000_011.extract_static(), 0b00_000_000);
    /// ```
    fn extract_static(self) -> u8 {
        (REGISTERS_STATIC_OPERAND_MASK & self) >> 3
    }

    /// Only first 3 bits are used.
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::RegistersEncoding;
    ///
    /// assert_eq!(0b10_011_111.set_static(0b111), 0b10_111_111);
    /// assert_eq!(0b11_011_100.set_static(0b101), 0b11_101_100);
    /// assert_eq!(0b00_010_010.set_static(0b101), 0b00_101_010);
    /// assert_eq!(0b01_000_011.set_static(0b000), 0b01_000_011);
    /// ```
    fn set_static(self, x_static: u8) -> u8 {
        let layer = (0b00000_111 & x_static) << 3;
        (!REGISTERS_STATIC_OPERAND_MASK & self) | layer
    }

    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::RegistersEncoding;
    ///
    /// assert_eq!(0b10_111_111.extract_dynamic(), 0b00_000_111);
    /// assert_eq!(0b11_100_101.extract_dynamic(), 0b00_000_101);
    /// assert_eq!(0b00_010_101.extract_dynamic(), 0b00_000_101);
    /// assert_eq!(0b01_011_000.extract_dynamic(), 0b00_000_000);
    /// ```
    fn extract_dynamic(self) -> u8 {
        REGISTERS_DYNAMIC_OPERAND_MASK & self
    }

    /// Only first 3 bits are used.
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::RegistersEncoding;
    ///
    /// assert_eq!(0b10_111_011.set_dynamic(0b111), 0b10_111_111);
    /// assert_eq!(0b11_100_011.set_dynamic(0b101), 0b11_100_101);
    /// assert_eq!(0b00_010_010.set_dynamic(0b101), 0b00_010_101);
    /// assert_eq!(0b01_011_000.set_dynamic(0b000), 0b01_011_000);
    /// ```
    fn set_dynamic(self, dynamic: u8) -> u8 {
        let layer = 0b00000_111 & dynamic;
        (!REGISTERS_DYNAMIC_OPERAND_MASK & self) | layer
    }
}
// endregion

/// Structure containing information about the operands of an instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data {
    /// Width of operands when dereferenced and for storing result.
    pub width: number::Size,
    /// The name of the operand to store the result of the computation in, if the computation produces a result. There
    /// is always a destination even if the instruction does not compute and store anything.
    pub destination: Destination,
    pub synchronous: bool,
    pub operands: Operands
}

#[derive(Debug)]
pub enum DataConstructError {
    /// Error caused when reading from stream.
    StreamRead(io::Error),
    /// Stream did not contain enough bytes.
    Length,
    /// Failed to construct the operands. This could be due to rule breaking or the operation trait is bad.
    Operands(OperandsConstructError),
    /// Invalid destination for the current addressing or operand modes. The current destination cannot be used. Reasons
    /// may include:
    /// - Dynamic destination was used with the constant addressing mode.
    /// - The dynamic operand is not expected, therefore there is no location to store the result at.
    ///
    /// This error is not produced if there are no operands because the destination is encoded as a boolean in the
    /// instruction.
    Destination
}

impl Data {
    /// Try to construct a data field from data with an operation and driver. The data structure contains information
    /// operands and how they should be handled and dealt with as well as addressing information for x_dynamic. This
    /// involves decoding the stream with [Registers].
    /// ```
    /// use std::io::Cursor;
    /// use atln_processor::emulator::processor::processor::instruction::{Data, Driver};
    /// use atln_processor::emulator::processor::processor::instruction::operation::arithmetic::Arithmetic;
    /// use atln_processor::emulator::processor::processor::instruction::operation::{Coded, Extension};
    /// use atln_processor::emulator::processor::processor::instruction::operand::Destination;
    ///
    /// let mut extension = Extension::Arithmetic(Arithmetic::Add);
    /// let extension_code = extension.code();
    ///
    /// let operation = extension.operation();
    /// let operation_code = operation.code();
    ///
    /// let data = Data::new(
    ///     &mut Cursor::new([ 00_000_000 ]),
    ///     operation,
    ///     &Driver {
    ///         extension: extension_code,
    ///         operation: operation_code,
    ///         addressing: 0,
    ///         dynamic_destination: false,
    ///         immediate_exponent: 0,
    ///         synchronise: false
    ///     }
    /// )
    ///     .unwrap();
    ///
    /// assert_eq!(data.destination, Destination::Static);
    /// ```
    pub fn new(stream: &mut impl Read, presence: &OperandsPresence, driver: &Driver) -> Result<Self, DataConstructError> {
        // Decode registers byte.
        let mut data_encoded = [0u8; 1];
        match stream.read(&mut data_encoded) {
            Ok(length) => if length != data_encoded.len() { return Err(DataConstructError::Length); },
            Err(error) => return Err(DataConstructError::StreamRead(error))
        };

        let registers = Registers::new(data_encoded[0]);
        let destination = if driver.dynamic_destination { Destination::Dynamic } else { Destination::Static };

        // operands extracting here
        let operands = match Operands::new(stream, presence, &registers, driver) {
            Ok(value) => value,
            Err(error) => return Err(DataConstructError::Operands(error))
        };

        // Prevent the invalid instruction configuration which involves pointing to a constant dynamic operand as the
        // destination operand.
        if let Some(x_dynamic) = operands.x_dynamic() { if let Destination::Dynamic = destination { if let Dynamic::Constant(_) = x_dynamic {
            return Err(DataConstructError::Destination);
        }}}

        // Construct data.
        Ok(Data {
            width: number::Size::from_exponent(registers.width).unwrap(),
            destination,
            synchronous: driver.synchronise,
            operands
        })
    }
}

#[derive(Debug, Default)]
pub struct Instruction {
    pub extension: Extension,
    pub data: Option<Data>
}

#[derive(Debug)]
pub enum InstructionConstructError {
    /// Stream failed to read.
    StreamRead(io::Error),
    /// Not enough bytes.
    Length,
    /// The extension and or operation are invalid.
    InvalidCode(ExtensionFromCodeInvalid),
    /// Failed to construct the data field of the instruction.
    Data(DataConstructError)
}

/// Caused by using a destination which corresponds to an operand that is not provided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationError {
    /// No data included.
    Data,
    /// The static operand wasn't present.
    Static,
    /// The dynamic operand wasn't present.
    Dynamic
}

impl Instruction {
    /// Use the driver, registers, and immediate to encode into a dynamic number of bytes. Encoding is variable
    /// length. The data is not validated here. To use an immediate, registers must be of the [Some] variant. If an
    /// immediate is [Some] and registers is [None] then [None] will also be returned.
    pub fn encode_driver_registers_immediate(driver: &mut Driver, registers: Option<&Registers>, immediate: Option<&number::Data>) -> Option<Vec<u8>> {
        let mut encoded = Vec::new();

        encoded.extend(driver.encode());
        if let Some(registers) = registers {
            encoded.push(registers.encode());
            if let Some(immediate) = immediate { encoded.extend(immediate.to_le_bytes()); }
        } else if immediate.is_some() { return None; }

        Some(encoded)
    }

    // Decode an encoded binary stream into a processor instruction. TODO: Tests
    pub fn new(stream: &mut impl Read) -> Result<Self, InstructionConstructError> {
        // Decode driver bytes.
        let mut encoded_driver = [0u8; 2];

        match stream.read(&mut encoded_driver) {
            Ok(length) => if length != encoded_driver.len() { return Err(InstructionConstructError::Length) },
            Err(error) => return Err(InstructionConstructError::StreamRead(error))
        };

        let driver = Driver::new(encoded_driver);

        let mut extension =  match Extension::from_codes(driver.extension, driver.operation) {
            Ok(operation) => operation,
            Err(error) => return Err(InstructionConstructError::InvalidCode(error))
        };

        // Decode data bytes.
        let operation = extension.operation();
        
        if let Some(presence) = operation.get_presence() {
            let data: Option<Data> = match Data::new(stream, &presence, &driver) {
                Ok(some) => Some(some),
                Err(error) => return Err(InstructionConstructError::Data(error))
            };

            // Construction
            return Ok(Self {
                extension,
                data
            })
        }
        
        Ok(Self {
            extension,
            data: None
        })
    }

    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::{Driver, Instruction, Registers};
    /// use atln_processor::emulator::processor::processor::instruction::operand::{CONSTANT_ADDRESSING, IMMEDIATE_EXPONENT_BYTE};
    /// use atln_processor::emulator::processor::processor::instruction::operation::arithmetic::ADD_CODE;
    /// use atln_processor::emulator::processor::processor::instruction::operation::ARITHMETIC_CODE;
    /// use atln_processor::number;
    /// 
    /// let mut driver = Driver {
    ///     extension: ARITHMETIC_CODE,
    ///     operation: ADD_CODE,
    ///     synchronise: true,
    ///     dynamic_destination: false,
    ///     addressing: CONSTANT_ADDRESSING,
    ///     immediate_exponent: IMMEDIATE_EXPONENT_BYTE
    /// };
    ///
    /// let registers = Registers {
    ///     width: IMMEDIATE_EXPONENT_BYTE,
    ///     x_static: 1,
    ///     x_dynamic: 0
    /// };
    ///
    /// let target = [ 0b000000_1_0, 0b0000_10_00, 0b00_001_000, 0b00001010 ];
    ///
    /// assert_eq!(Instruction::encode_driver_registers_immediate(&mut driver, Some(&registers), Some(&number::Data::Byte(10))).unwrap(), target);
    /// ```
    pub fn encode(&mut self) -> Vec<u8> {
        let mut synchronise = false;
        let mut dynamic_destination = false;
        let mut addressing = 0;
        let mut immediate_exponent = 0;
        let mut registers: Option<Registers> = None;
        let mut immediate: Option<number::Data> = None;

        if let Some(data) = &self.data {
            synchronise = data.synchronous;
            dynamic_destination = match data.destination {
                Destination::Dynamic => true,
                Destination::Static => false
            };

            let mut x_dynamic_code = 0;
            if let Some(x_dynamic) = data.operands.x_dynamic() {
                x_dynamic_code = x_dynamic.register().unwrap_or(0);
                immediate = x_dynamic.immediate().cloned();

                if let Some(immediate) = x_dynamic.immediate() { immediate_exponent = immediate.clone().exponent() }
                addressing = x_dynamic.addressing();
            }

            registers = Some(Registers {
                width: data.width.exponent(),
                x_static: data.operands.x_static().unwrap_or(0),
                x_dynamic: x_dynamic_code
            });
        }

        let mut driver = Driver {
            extension: self.extension.code(),
            operation: self.extension.operation().code(),
            synchronise,
            dynamic_destination,
            addressing,
            immediate_exponent
        };

        // Unwrapping should not fail because the processor is a controlled environment. There is no risk of an
        // immediate being present with a lack of [Registers]. Output of [encode_driver_registers_immediate] can safely
        // be unwrapped.
        if let Some(registers) = registers {
            if let Some(immediate) = immediate { Instruction::encode_driver_registers_immediate(&mut driver, Some(&registers), Some(&immediate)).unwrap() }
            else { Instruction::encode_driver_registers_immediate(&mut driver, Some(&registers), None).unwrap() }
        } else { Instruction::encode_driver_registers_immediate(&mut driver, None, None).unwrap() }
    }

    /// Get the operand that the destination property corresponds to.
    /// ```
    /// use atln_processor::emulator::processor::processor::instruction::{Data, Instruction, DestinationError};
    /// use atln_processor::emulator::processor::processor::instruction::operand::{AllPresent, Dynamic, Operands, Operand, Destination};
    /// use atln_processor::emulator::processor::processor::instruction::operation::arithmetic::Arithmetic;
    /// use atln_processor::emulator::processor::processor::instruction::operation::Extension;
    /// use atln_processor::number;
    ///
    /// let x_static = Instruction {
    ///     extension: Extension::Arithmetic(Arithmetic::Add),
    ///     data: Some(Data {
    ///         width: number::Size::Byte,
    ///         destination: Destination::Static,
    ///         synchronous: false,
    ///         operands: Operands::AllPresent(AllPresent {
    ///             x_static: 0,
    ///             x_dynamic: Dynamic::Register(1)
    ///         })
    ///     })
    /// };
    ///
    /// let x_dynamic = Instruction {
    ///     extension: Extension::Arithmetic(Arithmetic::Add),
    ///     data: Some(Data {
    ///         width: number::Size::Byte,
    ///         destination: Destination::Dynamic,
    ///         synchronous: false,
    ///         operands: Operands::AllPresent(AllPresent {
    ///             x_static: 0,
    ///             x_dynamic: Dynamic::Register(1)
    ///         })
    ///     })
    /// };
    ///
    /// let no_operands = Instruction {
    ///     extension: Extension::Arithmetic(Arithmetic::Add),
    ///     data: None
    /// };
    ///
    /// assert!(matches!(x_static.destination().unwrap(), Operand::Static(_)));
    /// assert!(!matches!(x_dynamic.destination().unwrap(), Operand::Static(_)));
    /// assert!(matches!(no_operands.destination(), Err(DestinationError::Data)));
    /// ```
    pub fn destination(&self) -> Result<Operand, DestinationError> {
        let data = match &self.data {
            Some(data) => data,
            None => return Err(DestinationError::Data)
        };

        Ok(match data.destination {
            Destination::Static => match data.operands.x_static() {
                Some(x_static) => Operand::Static(x_static),
                None => return Err(DestinationError::Static)
            },
            Destination::Dynamic => match data.operands.x_dynamic() {
                Some(x_dynamic) => Operand::Dynamic(x_dynamic.clone()),
                None => return Err(DestinationError::Dynamic)
            }
        })
    }
}