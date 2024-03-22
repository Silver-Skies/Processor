pub struct OperationParser {
    opcode: u8, 
    r0_expected: bool,
    r1_expected: bool,
    imm_expected: bool,
    imm_size: bool
}