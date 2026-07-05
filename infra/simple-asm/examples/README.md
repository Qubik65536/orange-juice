# Example Programs

Sample simple-asm programs demonstrating the language. Each file's header
comment describes exactly what the program does; this document adds context on
the behaviour each one exhibits.

Note that these examples only demonstrate interpreter behaviour (I/O, timing,
exit codes). How an external system such as an OJ judge classifies that
behaviour is entirely up to that system and is out of scope here.

| Program | Behaviour |
|---|---|
| `ac.sasm` | Reads an integer N, prints `N + 1`, exits 0. |
| `sum.sasm` | Reads a count N then N integers, prints their sum, exits 0. Demonstrates loops, conditional jumps, and repeated `READ`. |
| `wa.sasm` | Ignores stdin, prints the fixed string `wrong`, exits 0. |
| `tle_sleep.sasm` | Sleeps idly for 10 seconds (near-zero CPU usage), no output, then exits 0. |
| `tle_spin.sasm` | Infinite loop at 100% CPU; never reads, writes, or terminates. |
| `tle_burn.sasm` | Busy-loops for 10 seconds at 100% CPU, no output, then exits 0. |
| `re_exit.sasm` | Exits immediately with code 1, no output. |
| `re_div0.sasm` | Divides by zero; the interpreter prints a message to stderr and exits with the reserved code 101. |
| `partial_hang.sasm` | Prints `a` (no newline, flushed immediately), then sleeps for 10 seconds. Output written before the sleep is already visible even if the process is killed during it. |

## Running

```bash
echo 41 | cargo run -- examples/ac.sasm        # prints 42
printf '3\n1 2 3\n' | cargo run -- examples/sum.sasm  # prints 6
```
