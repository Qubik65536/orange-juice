//! simple-asm — a tiny assembly-language interpreter used to write small,
//! deterministic test programs for [benchy](../../benchy), the OJ submission
//! timer & checker.
//!
//! The pipeline is: read the source file, [`parser::parse`] it into a flat
//! instruction list, then [`exec::run`] it. See `SPEC.md` for the language
//! specification and `examples/` for sample programs (documented in
//! `examples/README.md`).
//!
//! # Usage
//!
//! ```text
//! simple-asm <program.sasm>
//! ```
//!
//! stdin, stdout, and stderr pass straight through to the running program.
//!
//! # Process exit codes
//!
//! | Code | Meaning |
//! |------|---------|
//! | 0    | `HALT` or end of program |
//! | n    | user-specified via `EXIT n` |
//! | 100  | parse error / unusable input file (reported before any execution) |
//! | 101  | runtime error (division by zero, `READ` at EOF, ...) |
//!
//! Codes 100 and 101 are reserved so callers can distinguish interpreter-level
//! failures from user-scripted `EXIT` codes.

mod exec;
mod instr;
mod parser;

/// Exit code for parse errors, bad CLI usage, or an unreadable program file.
const EXIT_PARSE_ERROR: i32 = 100;
/// Exit code for runtime errors (see [`exec::run`]).
const EXIT_RUNTIME_ERROR: i32 = 101;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: simple-asm <program.sasm>");
        std::process::exit(EXIT_PARSE_ERROR);
    }
    let src = std::fs::read_to_string(&args[1]).unwrap_or_else(|e| {
        eprintln!("simple-asm: failed to read '{}': {}", args[1], e);
        std::process::exit(EXIT_PARSE_ERROR);
    });
    let prog = parser::parse(&src).unwrap_or_else(|e| {
        eprintln!("simple-asm: parse error: {}", e);
        std::process::exit(EXIT_PARSE_ERROR);
    });
    match exec::run(&prog) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("simple-asm: runtime error: {}", e);
            std::process::exit(EXIT_RUNTIME_ERROR);
        }
    }
}
