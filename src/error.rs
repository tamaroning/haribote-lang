pub fn error_exit(s: String) -> ! {
    println!("Error: {}", s);
    std::process::exit(1);
}
