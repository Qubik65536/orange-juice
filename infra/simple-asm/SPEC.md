# simple-asm Language Specification

A tiny assembly-like language whose interpreter is used to write small,
deterministic programs for testing [benchy](../benchy), the OJ submission
timer & checker. The interpreter itself is judge-agnostic: it only runs
programs, and makes no assumptions about how their behaviour is classified.

## Design goals

1. **One line = one instruction** — no expressions, no nesting.
2. Every externally observable behaviour (stdin consumption, stdout/stderr
   output, elapsed time, exit code) has a **dedicated instruction**, so
   programs exhibiting a specific behaviour are 1–3 lines.
3. Deterministic: same input → same output, byte-for-byte.

## Syntax

```asm
; comment until end of line
label:              ; jump target
    MOV  r0, 5      ; mnemonics case-insensitive, operands comma-separated
```

- 8 integer registers `r0`–`r7` (signed 64-bit), all initialized to 0.
- 8 string registers `s0`–`s7`, all initialized to the empty string; usable only
  with `READ`, `PRINT`, and `PRINTLN`.
- Operands: register, integer literal, or double-quoted string
  (`PRINT`/`PRINTLN`/`EPRINT` only; escapes: `\n`, `\t`, `\"`, `\\`).

## Instruction set

### Core

| Instruction | Meaning |
|---|---|
| `MOV dst, src` | copy register/immediate into register |
| `ADD dst, src` | `dst += src` |
| `SUB dst, src` | `dst -= src` |
| `MUL dst, src` | `dst *= src` |
| `DIV dst, src` | `dst /= src`; division by zero is a runtime error (exit 101) |
| `JMP label` | unconditional jump |
| `JEQ a, b, label` | jump if `a == b` |
| `JNE a, b, label` | jump if `a != b` |
| `JLT a, b, label` | jump if `a < b` |
| `JGT a, b, label` | jump if `a > b` |

### I/O, timing, and termination

| Instruction | Meaning |
|---|---|
| `READ r` | reads the next whitespace-separated integer from stdin; EOF or a non-integer token is a runtime error |
| `READ s` | reads the next whitespace-separated token into a string register, like C `scanf("%s")` (skip leading whitespace, stop at next whitespace); EOF is a runtime error |
| `PRINT r` / `PRINT s` / `PRINT "str"` | writes to stdout, no trailing newline |
| `PRINTLN r` / `PRINTLN s` / `PRINTLN "str"` | writes to stdout with a trailing `\n` |
| `EPRINT "str"` | writes to stderr with a trailing `\n`; never touches stdout |
| `SLEEP ms` | idle-sleeps for `ms` milliseconds (wall-clock time, near-zero CPU time) |
| `BURN ms` | busy-loops for `ms` milliseconds (wall-clock *and* CPU time) |
| `EXIT code` | terminates the process with exit code `code` |
| `HALT` | exit 0; also implicit when execution falls off the end |

Stdout and stderr are flushed after every `PRINT*`/`EPRINT`, so output already
produced remains visible even if the process is killed externally
(e.g. `PRINT "a"` then `SLEEP 10000`).

## Exit codes

| Code | Meaning |
|---|---|
| `0` | `HALT` or end of program |
| `n` | user-specified via `EXIT n` |
| `100` | parse error (reported before any execution, message on stderr) |
| `101` | runtime error (division by zero, `READ` at EOF, etc.; message on stderr) |

Codes 100/101 are reserved so callers can distinguish interpreter-level
failures from user-scripted `EXIT` codes.

## Behaviour examples

Illustrative snippets showing how each kind of observable behaviour can be
produced. How an external system classifies these behaviours is out of scope
for this specification — see `examples/README.md` for full sample programs.

| Behaviour | Program |
|---|---|
| Read, compute, print, exit 0 | `READ r0` / `ADD r0, 1` / `PRINTLN r0` |
| Fixed output, exit 0 | `PRINTLN "wrong"` |
| Long idle sleep | `SLEEP 10000` |
| Never terminates / long CPU burn | `loop: JMP loop` or `BURN 10000` |
| Non-zero exit code | `EXIT 1` |
| Runtime error (exit 101) | `MOV r1, 0` / `DIV r0, r1` |
| Partial output, then hang | `PRINT "a"` / `SLEEP 10000` |

## Interpreter

- Rust binary, no runtime state outside the process.
- CLI: `simple-asm <program.sasm>` — stdin/stdout/stderr pass straight through;
  the process exit code follows the table above.
- Two-pass execution: pass 1 collects labels → instruction indices; pass 2
  executes a `Vec<Instr>` with a program counter.
- Parse errors report the offending line number.

## Deferred (v2, only if needed)

- `ALLOC kb` for memory-limit (MLE) testing
- Stack, `CALL`/`RET`
