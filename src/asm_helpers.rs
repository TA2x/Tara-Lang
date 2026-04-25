// src/asm_helpers.rs

/// Returns true if `ch` is an ASCII digit (0–9).
///
/// HOW THIS WORKS (Assembly explanation):
/// We load the character's ASCII byte value into a register,
/// subtract the value of '0' (48), compare against 9.
/// If the result is 0..=9, it's a digit.
///
/// `asm!` macro → Rust book: Appendix D / "Inline Assembly" (unstable book)
/// `#[inline(always)]` → tells the compiler to paste this function's code
/// directly at every call site — no function call overhead.
#[inline(always)]
pub fn is_digit_asm(ch: char) -> bool {
    // Guard: our asm only handles ASCII. For non-ASCII, fall back to Rust.
    if !ch.is_ascii() { return false; }

    let byte = ch as u8;
    let result: u8;

    // SAFETY: We are only reading CPU registers and performing arithmetic.
    // No memory is accessed, no invariants are violated.
    // `unsafe` is required for `asm!` → Rust book Chapter 19.1
    unsafe {
        core::arch::asm!(
            // al = byte (the char's ASCII value)
            // Subtract '0' (48). If byte < '0', this wraps to >9.
            "sub {byte}, 48",
            // Compare with 9. Sets carry flag if byte-48 > 9.
            "cmp {byte}, 9",
            // Set result byte to 1 if CF=0 (i.e., 0 <= byte-48 <= 9).
            // `setbe` = set if below or equal (unsigned compare).
            "setbe {result}",
            byte   = inout(reg_byte) byte => _,
            result = out(reg_byte) result,
            options(pure, nomem, nostack),
        );
    }

    result != 0
}

/// Returns true if `ch` is an ASCII letter (a–z, A–Z) or underscore.
///
/// ASSEMBLY STRATEGY:
/// We use two range checks:
///   lowercase: subtract 'a'(97), compare ≤ 25  → a..=z
///   uppercase: subtract 'A'(65), compare ≤ 25  → A..=Z
/// Underscore is checked with a direct compare.
/// The three results are OR-ed together.
#[inline(always)]
pub fn is_alpha_asm(ch: char) -> bool {
    if !ch.is_ascii() { return false; }

    // Underscore fast-path (pure Rust, no asm needed for one compare)
    if ch == '_' { return true; }

    let byte = ch as u8;
    let is_lower: u8;
    let is_upper: u8;

    unsafe {
        core::arch::asm!(
            // --- Lowercase check ---
            "mov {tmp}, {byte}",   // copy byte into tmp
            "sub {tmp}, 97",       // tmp = byte - 'a'
            "cmp {tmp}, 25",       // is tmp <= 25?
            "setbe {lo}",          // lo = 1 if yes

            // --- Uppercase check ---
            "mov {tmp}, {byte}",   // reload byte (sub modified tmp)
            "sub {tmp}, 65",       // tmp = byte - 'A'
            "cmp {tmp}, 25",       // is tmp <= 25?
            "setbe {hi}",          // hi = 1 if yes

            byte = in(reg_byte) byte,
            tmp  = out(reg_byte) _,   // scratch register, discarded
            lo   = out(reg_byte) is_lower,
            hi   = out(reg_byte) is_upper,
            options(pure, nomem, nostack),
        );
    }

    is_lower != 0 || is_upper != 0
}