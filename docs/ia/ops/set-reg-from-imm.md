### Set register from immediate
Used to set the value of a register directly from the instruction.
```
opcode  R0  IMM1
0       0   0
opcode  R0  IMM2
1       0   0
opcode  R0  IMM3
2       0   0
opcode  R0  IMM4
3       0   0
```
- R0: Target register to mutate.
- IMM: Source value to copy to the register.  