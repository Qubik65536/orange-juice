use std::process::Command;

/// Compiles `tests/fixtures/hello.rs` into a binary, runs benchy against it with the
/// bundled input/output fixtures, asserts exit code 0 (Accepted), then cleans
/// up the compiled binary.
#[test]
fn test_hello_accepted() {
    // Step 1: Compile the demo program.
    let compile = Command::new("rustc")
        .args(["tests/fixtures/hello.rs", "-o", "tests/fixtures/hello"])
        .output()
        .expect("Failed to invoke rustc");
    assert!(
        compile.status.success(),
        "rustc failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    // Step 2: Run benchy against the compiled binary.
    let result = Command::new(env!("CARGO_BIN_EXE_benchy"))
        .args([
            "-p",
            "tests/fixtures/hello",
            "-i",
            "tests/fixtures/input.txt",
            "-o",
            "tests/fixtures/output.txt",
        ])
        .output()
        .expect("Failed to run benchy");

    // Step 3: Clean up the compiled binary.
    let _ = std::fs::remove_file("tests/fixtures/hello");

    // Step 4: Assert Accepted (exit code 0).
    assert_eq!(
        result.status.code(),
        Some(0),
        "Expected exit code 0 (Accepted), got {:?}\nstdout: {}\nstderr: {}",
        result.status.code(),
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr),
    );
}
