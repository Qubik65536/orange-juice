//! Executor for parsed simple-asm programs.
//!
//! State is deliberately tiny: eight integer registers (`r0`–`r7`, signed
//! 64-bit, zero-initialized), eight string registers (`s0`–`s7`,
//! empty-initialized), and a program counter.
//!
//! Two I/O properties are guaranteed:
//!
//! - **stdout/stderr are flushed after every write**, so output already
//!   produced remains visible even if the process is killed externally.
//! - **stdin is read incrementally** (byte by byte through stdin's buffer),
//!   never with `read_to_end`, so interactive terminal input works and `READ`
//!   returns as soon as a complete token has arrived.

use std::io::{Read, Write};
use std::time::{Duration, Instant};

use crate::instr::{Instr, PrintArg, Val};

/// Reads the next whitespace-separated token from stdin, with C `scanf("%s")`
/// behaviour: skip leading whitespace, then consume up to the next whitespace
/// or EOF.
///
/// # Errors
///
/// Returns `Err` if stdin is at EOF before any token byte is seen, if the
/// token is not valid UTF-8, or on an I/O failure.
fn read_token() -> Result<String, String> {
    let stdin = std::io::stdin();
    let mut lock = stdin.lock();
    let mut byte = [0u8; 1];
    let mut token: Vec<u8> = Vec::new();
    loop {
        let n = lock
            .read(&mut byte)
            .map_err(|e| format!("failed to read stdin: {}", e))?;
        if n == 0 {
            // EOF
            if token.is_empty() {
                return Err("READ: no more input".to_string());
            }
            break;
        }
        if byte[0].is_ascii_whitespace() {
            if token.is_empty() {
                continue; // skip leading whitespace
            }
            break; // token complete
        }
        token.push(byte[0]);
    }
    String::from_utf8(token).map_err(|_| "READ: input is not valid UTF-8".to_string())
}

/// Reads the next whitespace-separated token from stdin and parses it as a
/// signed 64-bit integer (C `scanf("%lld")`-style token semantics).
///
/// # Errors
///
/// All [`read_token`] errors, plus a token that is not a valid integer.
fn read_int() -> Result<i64, String> {
    let token = read_token()?;
    token
        .parse::<i64>()
        .map_err(|_| format!("READ: '{}' is not an integer", token))
}

/// Runs a parsed program to completion.
///
/// Returns `Ok(exit_code)` when the program terminates normally: `0` for
/// `HALT` or falling off the end, or the value given to `EXIT`.
///
/// # Errors
///
/// Returns `Err` with a description on runtime errors — division by zero,
/// division overflow, `READ` failures, or write failures on stdout/stderr.
/// The caller maps this to the reserved exit code 101.
pub fn run(prog: &[Instr]) -> Result<i32, String> {
    let mut regs = [0i64; 8];
    let mut sregs: [String; 8] = Default::default();
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let mut pc = 0usize;

    /// Resolves a value operand against the integer registers.
    fn val(v: Val, regs: &[i64; 8]) -> i64 {
        match v {
            Val::Reg(r) => regs[r as usize],
            Val::Imm(n) => n,
        }
    }

    /// Renders a print argument to the exact text to be written.
    fn fmt(arg: &PrintArg, regs: &[i64; 8], sregs: &[String; 8]) -> String {
        match arg {
            PrintArg::Str(s) => s.clone(),
            PrintArg::SReg(r) => sregs[*r as usize].clone(),
            PrintArg::Val(v) => val(*v, regs).to_string(),
        }
    }

    while pc < prog.len() {
        match &prog[pc] {
            Instr::Mov(d, s) => regs[*d as usize] = val(*s, &regs),
            Instr::Add(d, s) => regs[*d as usize] = regs[*d as usize].wrapping_add(val(*s, &regs)),
            Instr::Sub(d, s) => regs[*d as usize] = regs[*d as usize].wrapping_sub(val(*s, &regs)),
            Instr::Mul(d, s) => regs[*d as usize] = regs[*d as usize].wrapping_mul(val(*s, &regs)),
            Instr::Div(d, s) => {
                let rhs = val(*s, &regs);
                if rhs == 0 {
                    return Err("division by zero".to_string());
                }
                regs[*d as usize] = regs[*d as usize]
                    .checked_div(rhs)
                    .ok_or_else(|| "division overflow".to_string())?;
            }
            // Jumps set the program counter directly and skip the pc increment.
            Instr::Jmp(t) => {
                pc = *t;
                continue;
            }
            Instr::Jeq(a, b, t) => {
                if val(*a, &regs) == val(*b, &regs) {
                    pc = *t;
                    continue;
                }
            }
            Instr::Jne(a, b, t) => {
                if val(*a, &regs) != val(*b, &regs) {
                    pc = *t;
                    continue;
                }
            }
            Instr::Jlt(a, b, t) => {
                if val(*a, &regs) < val(*b, &regs) {
                    pc = *t;
                    continue;
                }
            }
            Instr::Jgt(a, b, t) => {
                if val(*a, &regs) > val(*b, &regs) {
                    pc = *t;
                    continue;
                }
            }
            Instr::Read(r) => regs[*r as usize] = read_int()?,
            Instr::ReadStr(r) => sregs[*r as usize] = read_token()?,
            Instr::Print(arg) => {
                // Flush after every write so output is visible immediately,
                // even if the process is later killed externally.
                write!(stdout, "{}", fmt(arg, &regs, &sregs))
                    .and_then(|_| stdout.flush())
                    .map_err(|e| format!("failed to write stdout: {}", e))?;
            }
            Instr::Println(arg) => {
                writeln!(stdout, "{}", fmt(arg, &regs, &sregs))
                    .and_then(|_| stdout.flush())
                    .map_err(|e| format!("failed to write stdout: {}", e))?;
            }
            Instr::Eprint(s) => {
                writeln!(stderr, "{}", s)
                    .and_then(|_| stderr.flush())
                    .map_err(|e| format!("failed to write stderr: {}", e))?;
            }
            Instr::Sleep(ms) => {
                let ms = val(*ms, &regs).max(0) as u64;
                std::thread::sleep(Duration::from_millis(ms));
            }
            Instr::Burn(ms) => {
                // Busy-wait to consume real CPU time, unlike SLEEP.
                let ms = val(*ms, &regs).max(0) as u64;
                let end = Instant::now() + Duration::from_millis(ms);
                while Instant::now() < end {
                    std::hint::black_box(());
                }
            }
            Instr::Exit(code) => return Ok(val(*code, &regs) as i32),
            Instr::Halt => return Ok(0),
        }
        pc += 1;
    }
    Ok(0)
}
