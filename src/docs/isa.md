
TVM Instruction Set Architecture (ISA) — v0 (Prototype)

This document defines the initial bytecode instructions supported by TVM v0.

Encoding Rules
- Each instruction begins with a 1-byte opcode (u8).
- Some opcodes are followed by immediate operands.
- All immediate integers are little-endian.

Opcodes
0x01  PUSH_I64   Push a signed 64-bit integer onto the operand stack.
      Operands: i64 (8 bytes)
      Stack:    ... -> ..., i64

0x02  ADD        Pop two i64 values, push their sum.
      Operands: none
      Stack:    ..., a, b -> ..., (a + b)

0x03  SYSCALL    Invoke a host syscall (OS-facing operation).
      Operands: syscall_id (u8)
      Stack:    depends on syscall

0xFF  HALT       Stop execution successfully.
      Operands: none

Syscall IDs (v0)
0x01  PRINT_I64  Pop one i64 from the stack and print it to stdout.
      Stack: ..., x -> ...

Error Semantics (v0)
- Unknown opcode: runtime error (terminate with message)
- Stack underflow (missing operands): runtime error
- Type mismatch (non-i64 where i64 required): runtime error
- Unknown syscall id: runtime error
