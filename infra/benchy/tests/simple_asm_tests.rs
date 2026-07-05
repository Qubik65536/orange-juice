use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;

// =================================================================================================
// Integration tests that judge simple-asm example programs.
//
// The sibling `infra/simple-asm` crate ships a tiny interpreted language along with example
// programs covering every behaviour a submission can exhibit (correct output, wrong output,
// hanging, burning CPU, crashing). These tests run each example through benchy and assert
// the verdict exit code. The verdict expectations live here, on the judge side; the example
// programs themselves only describe their behaviour.
// =================================================================================================

/// Exit codes benchy reports to its caller (mirrors `exit_codes` in src/main.rs).
const ACCEPTED: i32 = 0;
const WRONG_ANSWER: i32 = 1;
const TIME_LIMIT_EXCEEDED: i32 = 2;
const RUNTIME_ERROR: i32 = 3;

/// Returns the simple-asm crate root, building the interpreter binary on first use.
fn simple_asm_dir() -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("benchy crate should live inside infra/")
        .join("simple-asm");

    static BUILD: Once = Once::new();
    BUILD.call_once(|| {
        let status = Command::new("cargo")
            .args(["build", "--quiet"])
            .current_dir(&dir)
            .status()
            .expect("Failed to invoke cargo to build simple-asm");
        assert!(status.success(), "Building simple-asm failed");
    });

    dir
}

/// Runs benchy on `examples/<example>.sasm` with the given stdin contents and expected
/// output, returning benchy's exit code.
fn judge(example: &str, input: &str, expected_output: &str, timeout_secs: u64) -> i32 {
    let sasm_dir = simple_asm_dir();
    let interpreter = sasm_dir.join("target/debug/simple-asm");
    let program = sasm_dir.join("examples").join(format!("{example}.sasm"));
    assert!(program.exists(), "Missing example program {program:?}");

    // Write the test case's input and expected output to per-example temp files.
    let tmp = std::env::temp_dir();
    let input_file = tmp.join(format!("benchy-sasm-{example}-input.txt"));
    let output_file = tmp.join(format!("benchy-sasm-{example}-output.txt"));
    std::fs::write(&input_file, input).expect("Failed to write input file");
    std::fs::write(&output_file, expected_output).expect("Failed to write expected output file");

    let result = Command::new(env!("CARGO_BIN_EXE_benchy"))
        .args([
            "-p",
            interpreter.to_str().unwrap(),
            "-a",
            program.to_str().unwrap(),
            "-i",
            input_file.to_str().unwrap(),
            "-o",
            output_file.to_str().unwrap(),
            "-t",
            &timeout_secs.to_string(),
        ])
        .output()
        .expect("Failed to run benchy");

    let _ = std::fs::remove_file(&input_file);
    let _ = std::fs::remove_file(&output_file);

    result.status.code().unwrap_or_else(|| {
        panic!(
            "benchy was killed by a signal\nstderr: {}",
            String::from_utf8_lossy(&result.stderr)
        )
    })
}

// -------------------------------------------------------------------------------------------------
// Accepted
// -------------------------------------------------------------------------------------------------

/// `ac.sasm` reads N and prints N + 1: matching output is Accepted.
#[test]
fn ac_program_is_accepted() {
    assert_eq!(judge("ac", "41\n", "42\n", 1), ACCEPTED);
}

/// `sum.sasm` reads a count then that many integers and prints their sum,
/// exercising loops and repeated READs end-to-end through benchy's stdin piping.
#[test]
fn sum_program_is_accepted() {
    assert_eq!(judge("sum", "3\n1 2 3\n", "6\n", 1), ACCEPTED);
}

/// A missing trailing newline in the expected-output file must not change the verdict.
#[test]
fn trailing_newline_is_ignored_in_comparison() {
    assert_eq!(judge("ac", "41\n", "42", 1), ACCEPTED);
}

// -------------------------------------------------------------------------------------------------
// Wrong Answer
// -------------------------------------------------------------------------------------------------

/// `wa.sasm` prints a fixed string regardless of input, so it never matches "42".
#[test]
fn wrong_output_is_wrong_answer() {
    assert_eq!(judge("wa", "41\n", "42\n", 1), WRONG_ANSWER);
}

// -------------------------------------------------------------------------------------------------
// Time Limit Exceeded
// -------------------------------------------------------------------------------------------------

/// `tle_sleep.sasm` idles for 10s (near-zero CPU): the wall-clock timer must still fire.
#[test]
fn sleeping_program_is_time_limit_exceeded() {
    assert_eq!(judge("tle_sleep", "", "", 1), TIME_LIMIT_EXCEEDED);
}

/// `tle_spin.sasm` loops forever at 100% CPU and never terminates on its own.
#[test]
fn spinning_program_is_time_limit_exceeded() {
    assert_eq!(judge("tle_spin", "", "", 1), TIME_LIMIT_EXCEEDED);
}

/// `tle_burn.sasm` busy-waits for 10s of real CPU time.
#[test]
fn burning_program_is_time_limit_exceeded() {
    assert_eq!(judge("tle_burn", "", "", 1), TIME_LIMIT_EXCEEDED);
}

/// `partial_hang.sasm` prints correct-looking partial output and then hangs:
/// the timeout must win even though some output was already produced.
#[test]
fn partial_output_then_hang_is_time_limit_exceeded() {
    assert_eq!(judge("partial_hang", "", "a", 1), TIME_LIMIT_EXCEEDED);
}

// -------------------------------------------------------------------------------------------------
// Runtime Error
// -------------------------------------------------------------------------------------------------

/// `re_exit.sasm` terminates with a non-zero exit code and no output.
#[test]
fn nonzero_exit_is_runtime_error() {
    assert_eq!(judge("re_exit", "", "", 1), RUNTIME_ERROR);
}

/// `re_div0.sasm` divides by zero; the interpreter exits with its reserved
/// runtime-error code (101), which benchy must classify as a Runtime Error.
#[test]
fn division_by_zero_is_runtime_error() {
    assert_eq!(judge("re_div0", "", "", 1), RUNTIME_ERROR);
}
