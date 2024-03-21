# Register manipulation operations

### Set register to immediate
Used to set the value of a register directly from the instruction.
```
opcode  R1  IMM
0       0   0
```
- R1: Target register to mutate.
- IMM: Source value to copy to the register.

### Copy register to register
Copy the contents from a source register to a target register.
```
opcode  R1  R2
1       0   0
```
- R1: Target register to mutate to assign the value coppied from the source register.
- R2: Source register to copy data from.
