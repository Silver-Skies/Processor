# Instruction encoding and formatting

### Dynamic parameters
The only required parameter for an instruction to be valid is the operation identifier and other parameters may be expected based on how the specific operation corrosponding to the operation code needs are. The order is always fixed as "Register 1", "Register 2", and "Immediate".
- Not all parameters are required by every instruction.
- Ordered as "Register 1", "Register 2", "Immediate".
- Dependent on operation being performed.

### Exploded instruction format
```
opcode r1 r2 imm
```

#### Operation Code (opcode)
Operation codes (opcodes) are 8 bit integers that can be used to reffer to the operation that the host must run.

#### Register parameters (r1 and r2)
Registers are of a variety of sizes up to the architecture bit size, volitile, and fast. They are referenced with 8 bit register identifiers. Certain instructions may expect a register identifier for performing operations. Instructions can accept at most 2 register parameters.
- Many sizes.
- 8 bits to reffer.
- 2 registers per instruction.

#### Immediate parameter (imm)
An immediate is typically is used to reffer to a memory address or store a large number. Instructions can accept one immediate parameter and is the size of the architecture bits. Some instructions may use the immediate parameter to reference a 3rd register.
- Size of architecture bit specification.
- Can be used to refer to register depending on the operation being performed.