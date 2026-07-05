# simple-asm

A simple assembly language interpreter, used to write small, deterministic test
programs for [benchy](../benchy) (the OJ submission timer & checker).

The language provides dedicated instructions for exercising each judge feature —
stdin feeding (`READ`), stdout output (`PRINT`/`PRINTLN`), timing (`SLEEP`/`BURN`),
and exit codes (`EXIT`) — so small, deterministic test programs are only a few
lines long. The interpreter itself is judge-agnostic: it only runs programs and
makes no assumptions about how their behaviour is classified.
See [SPEC.md](SPEC.md) for the full language specification.

## Setup

Make sure to build the project:

```bash
cargo build
```

## Usage

Run a program with:

```bash
cargo run -- <program.sasm>
```

Or invoke the built binary directly:

```bash
./target/debug/simple-asm examples/ac.sasm
```

The interpreter reads from stdin, writes to stdout/stderr, and exits with the
program's exit code (`0` for `HALT`/end of program, `n` for `EXIT n`).
Exit codes `100` (parse error) and `101` (runtime error) are reserved by the
interpreter itself.

## Examples

The [`examples/`](examples) directory contains sample programs demonstrating
the language and interpreter behaviour (I/O, timing, exit codes). Each program
is documented in [`examples/README.md`](examples/README.md).

## Testing

This project uses [Rust's built-in testing framework](https://doc.rust-lang.org/book/ch11-00-testing.html) for unit and integration tests.

Run all tests:

```bash
cargo test
```
