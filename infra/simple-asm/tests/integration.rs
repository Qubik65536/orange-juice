use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

const BIN: &str = env!("CARGO_BIN_EXE_simple-asm");

/// Runs the interpreter on the given program file, feeding `input` to stdin.
/// Returns (exit_code, stdout, stderr).
fn run_file(path: &std::path::Path, input: &str) -> (i32, String, String) {
    let mut child = Command::new(BIN)
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn simple-asm");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().expect("no exit code"),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

/// Runs an inline program by writing it to a uniquely-named temp file.
fn run_source(name: &str, source: &str, input: &str) -> (i32, String, String) {
    let path = std::env::temp_dir().join(format!(
        "simple-asm-test-{}-{}.sasm",
        std::process::id(),
        name
    ));
    std::fs::write(&path, source).unwrap();
    let result = run_file(&path, input);
    std::fs::remove_file(&path).ok();
    result
}

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

// =================================================================================================
// Example programs
// =================================================================================================

#[test]
fn example_ac() {
    let (code, stdout, _) = run_file(&example("ac.sasm"), "41\n");
    assert_eq!(code, 0);
    assert_eq!(stdout, "42\n");
}

#[test]
fn example_sum() {
    let (code, stdout, _) = run_file(&example("sum.sasm"), "4\n1 2 3 4\n");
    assert_eq!(code, 0);
    assert_eq!(stdout, "10\n");
}

#[test]
fn example_wa() {
    let (code, stdout, _) = run_file(&example("wa.sasm"), "");
    assert_eq!(code, 0);
    assert_eq!(stdout, "wrong\n");
}

#[test]
fn example_re_exit() {
    let (code, _, _) = run_file(&example("re_exit.sasm"), "");
    assert_eq!(code, 1);
}

#[test]
fn example_re_div0() {
    let (code, _, stderr) = run_file(&example("re_div0.sasm"), "");
    assert_eq!(code, 101);
    assert!(stderr.contains("division by zero"));
}

// =================================================================================================
// Language behavior
// =================================================================================================

#[test]
fn print_without_newline_and_immediates() {
    let (code, stdout, _) = run_source("print", "PRINT \"ab\"\nPRINT 5\nPRINT r0\n", "");
    assert_eq!(code, 0);
    assert_eq!(stdout, "ab50");
}

#[test]
fn eprint_goes_to_stderr_only() {
    let (code, stdout, stderr) = run_source("eprint", "EPRINT \"log\"\nPRINTLN \"out\"\n", "");
    assert_eq!(code, 0);
    assert_eq!(stdout, "out\n");
    assert_eq!(stderr, "log\n");
}

#[test]
fn exit_with_register_value() {
    let (code, _, _) = run_source("exitreg", "MOV r0, 7\nEXIT r0\n", "");
    assert_eq!(code, 7);
}

#[test]
fn halt_stops_execution() {
    let (code, stdout, _) = run_source("halt", "PRINTLN \"a\"\nHALT\nPRINTLN \"b\"\n", "");
    assert_eq!(code, 0);
    assert_eq!(stdout, "a\n");
}

#[test]
fn read_string_scanf_behaviour() {
    // Like C scanf("%s"): skips leading whitespace, stops at next whitespace.
    let src = "READ s0\nREAD r0\nREAD s1\nPRINTLN s0\nPRINTLN r0\nPRINTLN s1\n";
    let (code, stdout, _) = run_source("readstr", src, "  hello \n 42\tworld\n");
    assert_eq!(code, 0);
    assert_eq!(stdout, "hello\n42\nworld\n");
}

#[test]
fn read_string_past_eof_is_runtime_error() {
    let (code, _, stderr) = run_source("readstr_eof", "READ s0\nREAD s1\n", "one\n");
    assert_eq!(code, 101);
    assert!(stderr.contains("no more input"));
}

#[test]
fn read_past_eof_is_runtime_error() {
    let (code, _, stderr) = run_source("eof", "READ r0\nREAD r1\n", "1\n");
    assert_eq!(code, 101);
    assert!(stderr.contains("no more input"));
}

#[test]
fn parse_error_exits_100_before_any_output() {
    let (code, stdout, stderr) = run_source("parse", "PRINTLN \"a\"\nBOGUS r0\n", "");
    assert_eq!(code, 100);
    assert_eq!(stdout, "");
    assert!(stderr.contains("parse error"));
    assert!(stderr.contains("line 2"));
}

#[test]
fn sleep_takes_wall_time() {
    let start = Instant::now();
    let (code, _, _) = run_source("sleep", "SLEEP 200\n", "");
    assert_eq!(code, 0);
    assert!(start.elapsed().as_millis() >= 200);
}

#[test]
fn burn_takes_wall_time() {
    let start = Instant::now();
    let (code, _, _) = run_source("burn", "BURN 200\n", "");
    assert_eq!(code, 0);
    assert!(start.elapsed().as_millis() >= 200);
}

#[test]
fn negative_arithmetic_and_conditionals() {
    let src = "MOV r0, -5\nSUB r0, 3\nJLT r0, 0, neg\nPRINTLN \"non-negative\"\nHALT\nneg: PRINTLN r0\n";
    let (code, stdout, _) = run_source("neg", src, "");
    assert_eq!(code, 0);
    assert_eq!(stdout, "-8\n");
}
