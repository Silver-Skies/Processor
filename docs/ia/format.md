# Instruction encoding and formatting

### Dynamic parameters
The only required parameter for an instruction to be valid is the operation identifier and other parameters may be expected based on how the specific operation corrosponding to the operation code needs are. The order is always fixed as "Register 0", "Register 1", and "Immediate".
- Not all parameters are required by every instruction.
- Ordered as "Register 0", "Register 1", "Immediate".
- Dependent on operation being performed.

### Exploded instruction format
```
opcode r0 r1 imm
```

### Byte Quantisizations
This architecture quantisizes on units of 1, 2, 4, and 8 bytes.

#### Operation Code (opcode)
Operation codes (opcodes) are 8 bit integers that can be used to reffer to the operation that the host must run.
- Labeled as opcode.
- 1 byte to call.

#### Register parameters (r0 and r1)
Registers are of a variety of sizes up to the architecture bit size, volitile, and fast. They are referenced with 8 bit register identifiers. Certain instructions may expect a register identifier for performing operations. Instructions can accept at most 2 register parameters.
- Many sizes.
- 1 byte to reffer.
- 2 registers per instruction.
- Labeled as R with the index after.

#### Immediate parameter (imm)
An immediate is typically is used to reffer to a memory address or store a large number. Instructions can accept one immediate parameter and the size is determined by the operation. 

Some instructions may use the immediate parameter to reference a 3rd register. 

When an IMM is represented, its bytes are specified by them being listed right after the IMM. If no bytes are specified then it shows that its only being used as a label for the placeholder and is a generic example.
- Size is dependent on the operation.
- Can be used to refer to register depending on the operation being performed.
- Labeled as IMM with the number of bytes after and no bytes will be used to in situations where only order is being shown.