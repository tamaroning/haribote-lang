pub fn error_exit(s: String) -> ! {
    println!("Error: {}", s);
    panic!();
}
