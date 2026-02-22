Tara Virtual Machine (TVM) Bytecode Format — v0 (Prototype)

This document defines the initial `.tbc` file format used by TVM. The format is intentionally minimal for the OS-course prototype and will evolve via versioning.

Byte order: Little-endian for all multi-byte integers.

File Layout (v0)
1) Header (12 bytes total)
- magic      : 4 bytes  ASCII "TVM0"
- version    : u16      format version (v0 uses 1)
- reserved   : u16      must be 0 (reserved for future use)
- code_len   : u32      length of code section in bytes

2) Code Section (code_len bytes)
- code_bytes : [u8; code_len]  raw bytecode stream

Validation Rules
- File length must be at least 12 bytes.
- magic must equal "TVM0".
- version must be supported by the VM (v0 supports version=1).
- reserved must be 0.
- code_len must match the remaining file size:
  expected_total_size = 12 + code_len
  actual_total_size   = file_size
  Must satisfy: actual_total_size == expected_total_size

Notes
- v0 supports a single code stream intended to represent `main`.
- Future versions may add:
  - function tables
  - constant pools
  - debug info (line/column mapping)
  - metadata sections
