//! Parser for simple-asm source files.
//!
//! Parsing is line-based and happens in two passes:
//!
//! 1. **Pass 1** strips comments, collects `label:` definitions (mapping each
//!    label to the index of the next instruction), and gathers the remaining
//!    instruction lines together with their 1-based source line numbers.
//! 2. **Pass 2** parses each instruction line into an [`Instr`], resolving
//!    label references against the map built in pass 1. This is why forward
//!    jumps need no special handling.
//!
//! All errors are returned as `Err(String)` prefixed with `line N:` so the
//! caller can report them directly.

use std::collections::HashMap;

use crate::instr::{Instr, PrintArg, Val};

/// An operand token, produced by [`parse_operand`] before instruction-specific
/// validation (e.g. `MOV` requires its first operand to be a `Tok::Reg`).
#[derive(Debug, Clone, PartialEq)]
enum Tok {
    /// Integer register `r0`–`r7`.
    Reg(u8),
    /// String register `s0`–`s7`.
    SReg(u8),
    /// Integer literal.
    Imm(i64),
    /// String literal (escapes already resolved).
    Str(String),
    /// A bare identifier — only valid where a label is expected.
    Ident(String),
}

/// Strips a `;` comment from a line, ignoring semicolons that appear inside
/// string literals (e.g. `PRINT "a;b" ; real comment`).
fn strip_comment(line: &str) -> &str {
    let mut in_str = false;
    let mut esc = false;
    for (i, c) in line.char_indices() {
        if in_str {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
        } else if c == ';' {
            return &line[..i];
        }
    }
    line
}

/// If `s` starts with a `label:` definition, returns the label name and the
/// rest of the line (whitespace-trimmed). Label names are `[A-Za-z_][A-Za-z0-9_]*`.
///
/// Returns `None` if the line does not start with a label, which includes
/// ordinary instruction lines (they contain no top-level `:`).
fn take_label(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    if i > 0 && i < bytes.len() && bytes[i] == b':' && !bytes[0].is_ascii_digit() {
        Some((&s[..i], s[i + 1..].trim_start()))
    } else {
        None
    }
}

/// Parses a double-quoted string literal, resolving the escapes
/// `\n`, `\t`, `\"`, and `\\`. The literal must span the entire input.
fn parse_string(s: &str) -> Result<String, String> {
    let mut chars = s.chars();
    if chars.next() != Some('"') {
        return Err(format!("expected string literal, got '{}'", s));
    }
    let mut out = String::new();
    loop {
        match chars.next() {
            None => return Err("unterminated string literal".to_string()),
            Some('"') => break,
            Some('\\') => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(c) => return Err(format!("invalid escape '\\{}'", c)),
                None => return Err("unterminated string literal".to_string()),
            },
            Some(c) => out.push(c),
        }
    }
    if chars.next().is_some() {
        return Err(format!("unexpected characters after string literal: '{}'", s));
    }
    Ok(out)
}

/// Splits an operand list on top-level commas, respecting string literals
/// (a comma inside `"..."` does not split). Each part is whitespace-trimmed.
fn split_operands(s: &str) -> Result<Vec<&str>, String> {
    let mut parts = Vec::new();
    let mut in_str = false;
    let mut esc = false;
    let mut start = 0;
    for (i, c) in s.char_indices() {
        if in_str {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
        } else if c == ',' {
            parts.push(s[start..i].trim());
            start = i + 1;
        }
    }
    if in_str {
        return Err("unterminated string literal".to_string());
    }
    parts.push(s[start..].trim());
    Ok(parts)
}

/// Parses a single operand into a [`Tok`]. Recognized forms, in order:
/// string literal (`"..."`), register (`r0`–`r7` / `s0`–`s7`, case-insensitive),
/// integer literal, identifier (label reference).
fn parse_operand(s: &str) -> Result<Tok, String> {
    if s.is_empty() {
        return Err("empty operand".to_string());
    }
    if s.starts_with('"') {
        return Ok(Tok::Str(parse_string(s)?));
    }
    if s.len() == 2
        && let Some(is_string_reg) = match s.as_bytes()[0] {
            b'r' | b'R' => Some(false),
            b's' | b'S' => Some(true),
            _ => None,
        }
        && let Some(d) = s.chars().nth(1).and_then(|c| c.to_digit(10))
    {
        if d < 8 {
            return Ok(if is_string_reg {
                Tok::SReg(d as u8)
            } else {
                Tok::Reg(d as u8)
            });
        }
        return Err(format!("invalid register '{}' (valid: r0-r7, s0-s7)", s));
    }
    if let Ok(n) = s.parse::<i64>() {
        return Ok(Tok::Imm(n));
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if (first.is_ascii_alphabetic() || first == '_')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Ok(Tok::Ident(s.to_string()));
    }
    Err(format!("invalid operand '{}'", s))
}

// -------------------------------------------------------------------------------------------------
// Operand validation helpers: convert a Tok into the operand kind an
// instruction expects, or produce a descriptive error.
// -------------------------------------------------------------------------------------------------

/// Requires an integer register (`r0`–`r7`); returns its index.
fn expect_reg(t: &Tok) -> Result<u8, String> {
    match t {
        Tok::Reg(r) => Ok(*r),
        _ => Err(format!("expected register (r0-r7), got {:?}", t)),
    }
}

/// Requires an integer value: register or immediate.
fn expect_val(t: &Tok) -> Result<Val, String> {
    match t {
        Tok::Reg(r) => Ok(Val::Reg(*r)),
        Tok::Imm(n) => Ok(Val::Imm(*n)),
        _ => Err(format!("expected register or integer, got {:?}", t)),
    }
}

/// Requires a string literal.
fn expect_str(t: &Tok) -> Result<String, String> {
    match t {
        Tok::Str(s) => Ok(s.clone()),
        _ => Err(format!("expected string literal, got {:?}", t)),
    }
}

/// Requires an identifier naming a known label; returns the label's
/// instruction index.
fn expect_label(t: &Tok, labels: &HashMap<String, usize>) -> Result<usize, String> {
    match t {
        Tok::Ident(name) => labels
            .get(name)
            .copied()
            .ok_or_else(|| format!("unknown label '{}'", name)),
        _ => Err(format!("expected label, got {:?}", t)),
    }
}

/// Accepts anything printable: string literal, string register, integer
/// register, or immediate.
fn expect_print_arg(t: &Tok) -> Result<PrintArg, String> {
    match t {
        Tok::Str(s) => Ok(PrintArg::Str(s.clone())),
        Tok::SReg(r) => Ok(PrintArg::SReg(*r)),
        _ => Ok(PrintArg::Val(expect_val(t)?)),
    }
}

/// Parses a single instruction line (labels and comments already stripped)
/// into an [`Instr`], validating mnemonic, arity, and operand kinds.
fn parse_instr(text: &str, labels: &HashMap<String, usize>) -> Result<Instr, String> {
    // Split "MNEMONIC rest-of-line" on the first whitespace.
    let (mnem, rest) = match text.find(char::is_whitespace) {
        Some(i) => (&text[..i], text[i..].trim_start()),
        None => (text, ""),
    };
    let ops: Vec<Tok> = if rest.is_empty() {
        Vec::new()
    } else {
        split_operands(rest)?
            .iter()
            .map(|s| parse_operand(s))
            .collect::<Result<_, _>>()?
    };
    let mnem_up = mnem.to_ascii_uppercase();
    let arity = |n: usize| -> Result<(), String> {
        if ops.len() == n {
            Ok(())
        } else {
            Err(format!(
                "{} expects {} operand(s), got {}",
                mnem_up,
                n,
                ops.len()
            ))
        }
    };
    match mnem_up.as_str() {
        "MOV" => arity(2).and_then(|_| Ok(Instr::Mov(expect_reg(&ops[0])?, expect_val(&ops[1])?))),
        "ADD" => arity(2).and_then(|_| Ok(Instr::Add(expect_reg(&ops[0])?, expect_val(&ops[1])?))),
        "SUB" => arity(2).and_then(|_| Ok(Instr::Sub(expect_reg(&ops[0])?, expect_val(&ops[1])?))),
        "MUL" => arity(2).and_then(|_| Ok(Instr::Mul(expect_reg(&ops[0])?, expect_val(&ops[1])?))),
        "DIV" => arity(2).and_then(|_| Ok(Instr::Div(expect_reg(&ops[0])?, expect_val(&ops[1])?))),
        "JMP" => arity(1).and_then(|_| Ok(Instr::Jmp(expect_label(&ops[0], labels)?))),
        "JEQ" | "JNE" | "JLT" | "JGT" => {
            arity(3)?;
            let a = expect_val(&ops[0])?;
            let b = expect_val(&ops[1])?;
            let target = expect_label(&ops[2], labels)?;
            Ok(match mnem_up.as_str() {
                "JEQ" => Instr::Jeq(a, b, target),
                "JNE" => Instr::Jne(a, b, target),
                "JLT" => Instr::Jlt(a, b, target),
                _ => Instr::Jgt(a, b, target),
            })
        }
        "READ" => {
            arity(1)?;
            match &ops[0] {
                Tok::Reg(r) => Ok(Instr::Read(*r)),
                Tok::SReg(r) => Ok(Instr::ReadStr(*r)),
                t => Err(format!("expected register, got {:?}", t)),
            }
        }
        "PRINT" => arity(1).and_then(|_| Ok(Instr::Print(expect_print_arg(&ops[0])?))),
        "PRINTLN" => arity(1).and_then(|_| Ok(Instr::Println(expect_print_arg(&ops[0])?))),
        "EPRINT" => arity(1).and_then(|_| Ok(Instr::Eprint(expect_str(&ops[0])?))),
        "SLEEP" => arity(1).and_then(|_| Ok(Instr::Sleep(expect_val(&ops[0])?))),
        "BURN" => arity(1).and_then(|_| Ok(Instr::Burn(expect_val(&ops[0])?))),
        "EXIT" => arity(1).and_then(|_| Ok(Instr::Exit(expect_val(&ops[0])?))),
        "HALT" => arity(0).map(|_| Instr::Halt),
        _ => Err(format!("unknown instruction '{}'", mnem)),
    }
}

/// Parses a full simple-asm program into an executable instruction list.
///
/// # Errors
///
/// Returns `Err` with a `line N:`-prefixed message on the first syntax error
/// encountered: unknown mnemonics, wrong arity or operand kinds, malformed
/// literals, duplicate labels, or references to undefined labels.
pub fn parse(src: &str) -> Result<Vec<Instr>, String> {
    // Pass 1: strip comments/labels, record label -> instruction index.
    let mut instr_lines: Vec<(usize, &str)> = Vec::new();
    let mut labels: HashMap<String, usize> = HashMap::new();
    for (i, raw) in src.lines().enumerate() {
        let lineno = i + 1;
        let mut rest = strip_comment(raw).trim();
        while let Some((label, r)) = take_label(rest) {
            if labels.insert(label.to_string(), instr_lines.len()).is_some() {
                return Err(format!("line {}: duplicate label '{}'", lineno, label));
            }
            rest = r;
        }
        if !rest.is_empty() {
            instr_lines.push((lineno, rest));
        }
    }
    // Pass 2: parse instructions with all labels known.
    instr_lines
        .iter()
        .map(|(lineno, text)| {
            parse_instr(text, &labels).map_err(|e| format!("line {}: {}", lineno, e))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_comments_outside_strings() {
        assert_eq!(strip_comment("MOV r0, 1 ; hello"), "MOV r0, 1 ");
        assert_eq!(strip_comment("PRINT \"a;b\" ; c"), "PRINT \"a;b\" ");
        assert_eq!(strip_comment("no comment"), "no comment");
    }

    #[test]
    fn parses_operands() {
        assert_eq!(parse_operand("r3").unwrap(), Tok::Reg(3));
        assert_eq!(parse_operand("R7").unwrap(), Tok::Reg(7));
        assert_eq!(parse_operand("s2").unwrap(), Tok::SReg(2));
        assert_eq!(parse_operand("S0").unwrap(), Tok::SReg(0));
        assert!(parse_operand("s8").is_err());
        assert_eq!(parse_operand("-42").unwrap(), Tok::Imm(-42));
        assert_eq!(
            parse_operand("\"a\\n\"").unwrap(),
            Tok::Str("a\n".to_string())
        );
        assert_eq!(
            parse_operand("loop").unwrap(),
            Tok::Ident("loop".to_string())
        );
        assert!(parse_operand("r9").is_err());
        assert!(parse_operand("\"unterminated").is_err());
    }

    #[test]
    fn splits_operands_respecting_strings() {
        assert_eq!(
            split_operands("r0, \"a, b\", 5").unwrap(),
            vec!["r0", "\"a, b\"", "5"]
        );
    }

    #[test]
    fn parses_program_with_labels() {
        let src = "; sum\nMOV r0, 0\nloop: ADD r0, 1\nJLT r0, 3, loop\nPRINTLN r0\n";
        let prog = parse(src).unwrap();
        assert_eq!(prog[0], Instr::Mov(0, Val::Imm(0)));
        assert_eq!(prog[1], Instr::Add(0, Val::Imm(1)));
        assert_eq!(prog[2], Instr::Jlt(Val::Reg(0), Val::Imm(3), 1));
    }

    #[test]
    fn rejects_bad_programs() {
        assert!(parse("BOGUS r0").unwrap_err().contains("line 1"));
        assert!(parse("JMP nowhere").unwrap_err().contains("unknown label"));
        assert!(
            parse("x: HALT\nx: HALT")
                .unwrap_err()
                .contains("duplicate label")
        );
        assert!(parse("MOV 1, 2").unwrap_err().contains("expected register"));
        assert!(parse("HALT r0").unwrap_err().contains("expects 0 operand"));
        assert!(parse("MOV s0, 1").unwrap_err().contains("expected register"));
        assert!(parse("ADD s0, 1").unwrap_err().contains("expected register"));
    }

    #[test]
    fn read_accepts_both_register_kinds() {
        assert_eq!(parse("READ r0").unwrap()[0], Instr::Read(0));
        assert_eq!(parse("READ s5").unwrap()[0], Instr::ReadStr(5));
        assert!(parse("READ 3").unwrap_err().contains("expected register"));
    }

    #[test]
    fn mnemonics_are_case_insensitive() {
        assert!(parse("mov r0, 1\nhalt").is_ok());
    }
}
