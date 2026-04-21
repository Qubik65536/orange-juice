# Compile the demo program `hello.rs`.
rustc hello.rs -o hello

# Run the benchy program with the compiled `hello` binary as an argument.
# The `--release` flag is used to compile the benchy program in release mode,
# which optimizes the code for better performance.
cargo run --release -- -p hello -i input.txt -o output.txt

# Clean up the compiled `hello` binary after the test is done.
rm hello
