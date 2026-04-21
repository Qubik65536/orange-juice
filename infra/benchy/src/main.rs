use clap::Parser;
use std::os::unix::fs::PermissionsExt;

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
        // .stdout(std::process::Stdio::piped())
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
    if let Some(mut stdin) = child.stdin.take() {
        let input_data = std::fs::read(&args.input_file).unwrap_or_else(|e| {
            eprintln!(
                "Error: Failed to read input file '{}': {}",
                args.input_file, e
            );
            std::process::exit(exit_codes::JUDGE_ERROR);
        });
        std::io::copy(&mut &input_data[..], &mut stdin).unwrap_or_else(|e| {
            eprintln!("Error: Failed to write to program's stdin: {}", e);
            std::process::exit(exit_codes::JUDGE_ERROR);
        });
    } else {
        eprintln!("Error: Failed to capture program's stdin.");
        std::process::exit(exit_codes::JUDGE_ERROR);
    }
}
