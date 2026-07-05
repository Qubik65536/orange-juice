use clap::Parser;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use wait_timeout::ChildExt;

// =================================================================================================
// Exit code constants — using an enum-like module for clarity.
// These codes are what the *judge* process returns to the caller
// (e.g. Docker, the OJ backend, or a CI pipeline).
// =================================================================================================
mod exit_codes {
    /// The submitted program produced correct output within the time limit.
    pub const ACCEPTED: i32 = 0;
    /// The submitted program's output does not match the expected output.
    pub const WRONG_ANSWER: i32 = 1;
    /// The submitted program did not finish within the allowed time.
    pub const TIME_LIMIT_EXCEEDED: i32 = 2;
    /// The submitted program crashed or returned a non-zero exit code.
    pub const RUNTIME_ERROR: i32 = 3;
    /// Something went wrong inside the judge itself (I/O error, bad args, etc.).
    pub const JUDGE_ERROR: i32 = 4;
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Relative Path to the binary to be executed.
    #[arg(short = 'p', long)]
    program: String,

    /// Arguments to be passed to the program, separated by spaces. (Optional)
    #[arg(short = 'a', long, default_value = "")]
    program_args: String,

    /// Relative Path to the File Containing Inputs to be fed to the program.
    #[arg(short = 'i', long)]
    input_file: String,

    /// Relative Path to the File Containing Outputs to be compared with the program's output.
    #[arg(short = 'o', long)]
    output_file: String,

    /// Time Limit for the execution of the program in seconds.
    #[arg(short = 't', long, default_value_t = 1)]
    timeout: u64,
}

fn main() {
    // Get command line arguments.
    let args = Args::parse();

    // If the program path is relative and doesn't start with "./" or "/", prepend "./"
    // so that Command::new finds it in the current directory instead of searching PATH.
    let program_path = if !args.program.starts_with('/') && !args.program.starts_with("./") {
        format!("./{}", args.program)
    } else {
        args.program.clone()
    };

    // ---------------------------------------------------------------------------------------------
    // Step 01: Make sure the program exists and is executable.
    // ---------------------------------------------------------------------------------------------
    if !std::path::Path::new(&program_path).exists() {
        eprintln!("Error: Program '{}' does not exist.", program_path);
        std::process::exit(exit_codes::JUDGE_ERROR);
    }
    if !std::fs::metadata(&program_path)
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
    {
        eprintln!("Error: Program '{}' is not executable.", program_path);
        std::process::exit(exit_codes::JUDGE_ERROR);
    }

    // ---------------------------------------------------------------------------------------------
    // Step 02: Check the existence of input and output files.
    // ---------------------------------------------------------------------------------------------
    if !std::path::Path::new(&args.input_file).exists() {
        eprintln!("Error: Input file '{}' does not exist.", args.input_file);
        std::process::exit(exit_codes::JUDGE_ERROR);
    }
    if !std::path::Path::new(&args.output_file).exists() {
        eprintln!("Error: Output file '{}' does not exist.", args.output_file);
        std::process::exit(exit_codes::JUDGE_ERROR);
    }

    // ---------------------------------------------------------------------------------------------
    // Step 03: Spawn a child process to execute the program.
    // ---------------------------------------------------------------------------------------------
    // We pipe the stdin, stdout, and stderr of the child process
    // so we can feed it input and capture its output.
    let mut child = match std::process::Command::new(&program_path)
        .args(args.program_args.split_whitespace())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Error: Failed to spawn program '{}': {}", program_path, e);
            std::process::exit(exit_codes::JUDGE_ERROR);
        }
    };

    // ---------------------------------------------------------------------------------------------
    // Step 04: Feed the input file to the child process.
    // ---------------------------------------------------------------------------------------------
    // The write runs on a background thread so a program that fills its output pipe before
    // consuming all of its input cannot deadlock the judge. Dropping the handle when the
    // thread finishes closes the program's stdin (EOF). A write error here is not fatal:
    // a program that exits without reading its input closes the pipe, which is its right.
    let input_data = std::fs::read(&args.input_file).unwrap_or_else(|e| {
        eprintln!(
            "Error: Failed to read input file '{}': {}",
            args.input_file, e
        );
        std::process::exit(exit_codes::JUDGE_ERROR);
    });
    if let Some(mut stdin) = child.stdin.take() {
        std::thread::spawn(move || {
            let _ = std::io::copy(&mut &input_data[..], &mut stdin);
        });
    } else {
        eprintln!("Error: Failed to capture program's stdin.");
        std::process::exit(exit_codes::JUDGE_ERROR);
    }

    // ---------------------------------------------------------------------------------------------
    // Step 05: Drain the program's stdout/stderr on background threads.
    // ---------------------------------------------------------------------------------------------
    // Reading on separate threads keeps the pipe buffers from filling up: a program that writes
    // more output than the OS pipe buffer holds would otherwise block forever and be judged TLE.
    let mut stdout_pipe = child.stdout.take().unwrap_or_else(|| {
        eprintln!("Error: Failed to capture program's stdout.");
        std::process::exit(exit_codes::JUDGE_ERROR);
    });
    let stdout_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buf);
        buf
    });
    let mut stderr_pipe = child.stderr.take().unwrap_or_else(|| {
        eprintln!("Error: Failed to capture program's stderr.");
        std::process::exit(exit_codes::JUDGE_ERROR);
    });
    let stderr_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buf);
        buf
    });

    // ---------------------------------------------------------------------------------------------
    // Step 06: Record the start time for timer and wait for the child process to finish,
    //          killing it if it exceeds the time limit.
    // ---------------------------------------------------------------------------------------------
    let start_time = std::time::Instant::now();
    let timeout_duration = std::time::Duration::from_secs(args.timeout);
    let status = match child.wait_timeout(timeout_duration) {
        Ok(Some(status)) => status,
        Ok(None) => {
            // Still running past the deadline: kill it and report TLE.
            let _ = child.kill();
            let _ = child.wait();
            eprintln!("Time Limit Exceeded: program ran longer than {}s.", args.timeout);
            std::process::exit(exit_codes::TIME_LIMIT_EXCEEDED);
        }
        Err(e) => {
            eprintln!("Error: Failed to wait for program '{}': {}", program_path, e);
            std::process::exit(exit_codes::JUDGE_ERROR);
        }
    };
    let elapsed = start_time.elapsed();
    eprintln!("Time: {} ms", elapsed.as_millis());

    // ---------------------------------------------------------------------------------------------
    // Step 07: Check how the program terminated. A non-zero exit code or a signal
    //          (e.g. SIGSEGV) is a Runtime Error.
    // ---------------------------------------------------------------------------------------------
    let program_stdout = stdout_thread.join().unwrap_or_default();
    let program_stderr = stderr_thread.join().unwrap_or_default();
    if !status.success() {
        match status.code() {
            Some(code) => eprintln!("Runtime Error: program exited with code {}.", code),
            None => eprintln!("Runtime Error: program was terminated by a signal."),
        }
        if !program_stderr.is_empty() {
            eprintln!("Program stderr:\n{}", String::from_utf8_lossy(&program_stderr));
        }
        std::process::exit(exit_codes::RUNTIME_ERROR);
    }

    // ---------------------------------------------------------------------------------------------
    // Step 08: Compare the program's output with the expected output.
    // ---------------------------------------------------------------------------------------------
    let expected = std::fs::read(&args.output_file).unwrap_or_else(|e| {
        eprintln!(
            "Error: Failed to read output file '{}': {}",
            args.output_file, e
        );
        std::process::exit(exit_codes::JUDGE_ERROR);
    });
    if normalize_output(&program_stdout) == normalize_output(&expected) {
        std::process::exit(exit_codes::ACCEPTED);
    } else {
        eprintln!("Wrong Answer: output does not match expected output.");
        std::process::exit(exit_codes::WRONG_ANSWER);
    }
}

// =================================================================================================
// Output comparison
// =================================================================================================

/// Normalizes program output for comparison: trailing whitespace on each line is ignored,
/// as are trailing blank lines, so a missing or extra final newline does not affect the verdict.
fn normalize_output(bytes: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(bytes);
    let mut lines: Vec<String> = text.lines().map(|l| l.trim_end().to_string()).collect();
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines
}
