# Memory security policy (MSP)
Not all instructions are able to run due to lack of permissions.

### Permission modes
SSPIA supports unguarded and guarded mode.

The architecture will start out with unguarded mode on startup. The permission mode is stored in the permmode register and can be changed only if the host is currently in unguarded mode.
- Supports guarded and unguarded mode.
- Used to restrict running certain operations typically for when executing programs that arn't related to the main program.

### Guarded mode
Guarded permission modes rejects guarded operations from running successfully. Operations that are guarded will state that they require unguarded mode to execute. 

### Unguarded mode
In unguarded mode all operations can be executed without being rejected for security regarding permission modes.