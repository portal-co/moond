# Registers (A, Q, Z, LP, B, G, S, SQ) — Modernized reference

Source: `agcis_3_central_processor.pdf` — pp.6–12, table 3-1.

Summary
- Describes the main CP registers, typical uses, and simple primitives to read/write them in emulation.

Register semantics
- `A` (Accumulator): primary arithmetic register. Preserves data between instructions.
- `Q`: secondary register — stores return address for `TC K`, holds dividend/remainder for `DV K`.
- `Z` (Program Counter): address of next BASIC instruction to execute.
- `LP` (Low-Product): low-order product storage used by `MP K` and `DV K`.
- `B`: holds the *next* instruction word that will be executed; used heavily by NDX and interrupt sequences.
- `G`: memory buffer register — interface between CP and memory sense amps; used for reads/writes.
- `S`: staging register for memory address during STMIC cycles.
- `SQ`: 4-bit order-code register (feeds Sequence Generator). Holds the current order code to execute.

Pseudocode access helpers and canonical types

```c
// Canonical 15-bit types used in these docs
typedef int16_t  int15_t;   // signed 15-bit value (stored in 16-bit container)
typedef uint16_t uint15_t;  // unsigned 15-bit value (low 15 bits used)

// Read / write helpers — atomic for documentation purposes
uint15_t read_register(const char *rname);
void write_register(const char *rname, uint15_t value);

// Helper: sign-extend a 15-bit AGC word into a 32-bit signed value
static inline int32_t sign_extend15(uint15_t v) {
    // AGC uses 15-bit magnitude with sign in bit 15; this helper normalizes to int32
    if (v & 0x4000) { // sign bit set (assuming bit 14 is sign in 0-based indexing)
        return (int32_t)(v | 0xFFFF8000); // extend negative
    }
    return (int32_t)(v & 0x7FFF);
}

// Example: fetch order code from B (canonical helper used across docs)
static inline uint8_t extract_order_code(uint15_t B_word) {
    return (uint8_t)((B_word >> 11) & 0xF); // top 4 bits as example mapping; see ref/Instruction.md
}

// Example: set the program counter
static inline void set_PC(uint15_t addr) {
    write_register("Z", addr);
}
```

Register I/O notes
- Register G write gating is address dependent (`WG1G..WG6G`) allowing shifting/cycling into different bit positions (pp.15–18, table 3‑3).
- Register B supports both normal and complemented read gates; useful for certain micro-ops that require complemented instruction words (pp.18–19).

Citations
- AGCIS Issue 3, pp.6–12, table 3‑1, figs. 3‑2, 3‑3.