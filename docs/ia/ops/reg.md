# Register manipulation operations

### Set register to immediate
Used to set the value of a register directly from the instruction.
```
opcode  R0  IMM
0       0   0
```
- R0: Target register to mutate.
- IMM: Source value to copy to the register.

### Copy register to register
Copy the contents from a source register to a target register.
```
opcode  R0  R1
1       0   0
```
- R0: Target register to mutate to assign the value coppied from the source register.
- R1: Source register to copy data from.
