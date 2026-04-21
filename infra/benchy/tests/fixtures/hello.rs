fn main() {
    // 1. Print "Hello, World!".
    println!("Hello, World!");

    let stdin = std::io::stdin();
    let mut stdin_lock = stdin.lock();

    // 2. Read a string from stdin.
    let mut input = String::new();
    std::io::BufRead::read_line(&mut stdin_lock, &mut input).expect("Failed to read line");

    // 3. Print the read string to stdout.
    print!("{}", input);
    println!();
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    // 4. Read an int from stdin.
    let mut num_str = String::new();
    std::io::BufRead::read_line(&mut stdin_lock, &mut num_str).expect("Failed to read line");
    let num: i32 = num_str.trim().parse().expect("Failed to parse integer");

    // 5. Wait 100ms.
    std::thread::sleep(std::time::Duration::from_millis(100));

    // 6. Print (num + 65536) % 56.
    println!("{}", (num + 65536) % 56);
}
