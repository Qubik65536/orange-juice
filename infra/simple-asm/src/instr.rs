//! Instruction set definitions for the simple-asm language.
//!
//! A parsed program is a flat `Vec<Instr>`. Jump targets are resolved to
//! instruction indices at parse time, so the executor never sees label names.
//! See `SPEC.md` for the source-level syntax of each instruction.

/// A value operand: either an integer register index (`r0`–`r7`) or an
/// immediate signed 64-bit integer literal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Val {
    /// The current contents of integer register `rN` (index 0–7).
    Reg(u8),
    /// A literal integer value.
    Imm(i64),
}

/// Argument to `PRINT` / `PRINTLN`.
#[derive(Debug, Clone, PartialEq)]
pub enum PrintArg {
    /// An integer value (register or immediate), printed in decimal.
    Val(Val),
    /// The contents of string register `sN` (index 0–7).
    SReg(u8),
    /// A string literal, printed verbatim (escapes already resolved).
    Str(String),
}

/// One executable instruction.
///
/// Register operands are stored as raw indices (0–7); validation happens at
/// parse time, so the executor can index register arrays without checks.
#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    /// `MOV dst, src` — copy `src` into integer register `dst`.
    Mov(u8, Val),
    /// `ADD dst, src` — `dst += src` (wrapping on overflow).
    Add(u8, Val),
    /// `SUB dst, src` — `dst -= src` (wrapping on overflow).
    Sub(u8, Val),
    /// `MUL dst, src` — `dst *= src` (wrapping on overflow).
    Mul(u8, Val),
    /// `DIV dst, src` — `dst /= src` (truncating). Division by zero and
    /// `i64::MIN / -1` overflow are runtime errors.
    Div(u8, Val),
    /// `JMP label` — unconditional jump to the resolved instruction index.
    Jmp(usize),
    /// `JEQ a, b, label` — jump if `a == b`.
    Jeq(Val, Val, usize),
    /// `JNE a, b, label` — jump if `a != b`.
    Jne(Val, Val, usize),
    /// `JLT a, b, label` — jump if `a < b`.
    Jlt(Val, Val, usize),
    /// `JGT a, b, label` — jump if `a > b`.
    Jgt(Val, Val, usize),
    /// `READ rN` — read the next whitespace-separated integer from stdin.
    /// EOF or a non-integer token is a runtime error.
    Read(u8),
    /// `READ sN` — read the next whitespace-separated token from stdin into a
    /// string register, like C `scanf("%s")`. EOF is a runtime error.
    ReadStr(u8),
    /// `PRINT arg` — write `arg` to stdout without a trailing newline,
    /// flushing immediately.
    Print(PrintArg),
    /// `PRINTLN arg` — write `arg` to stdout with a trailing newline,
    /// flushing immediately.
    Println(PrintArg),
    /// `EPRINT "str"` — write a string literal to stderr with a trailing
    /// newline, flushing immediately. Never touches stdout.
    Eprint(String),
    /// `SLEEP ms` — idle-sleep for `ms` milliseconds (wall-clock time,
    /// negligible CPU time). Negative values are treated as 0.
    Sleep(Val),
    /// `BURN ms` — busy-loop for `ms` milliseconds (wall-clock *and* CPU
    /// time). Negative values are treated as 0.
    Burn(Val),
    /// `EXIT code` — terminate the process with the given exit code.
    Exit(Val),
    /// `HALT` — terminate the process with exit code 0. Also implied when
    /// execution falls off the end of the program.
    Halt,
}
