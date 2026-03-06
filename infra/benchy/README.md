# Benchy

This is the Rust-powered submission timer & checker program for the OJ system.

## Setup

Make sure to install the dependencies:

```bash
cargo build
```

## Running the Server

This program is designed to be run on in a Linux-based Docker container environment. Build the Docker image using the provided `Dockerfile` and run the container:

```bash
docker build -t benchy .
docker run -d --name benchy-container benchy
```

## Testing

This project uses [Rust's built-in testing framework](https://doc.rust-lang.org/book/ch11-00-testing.html) for unit and integration tests.

Run all tests:

```bash
cargo test
```
