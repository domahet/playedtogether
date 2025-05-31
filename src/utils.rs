pub fn print_in_box(lines: &[&str]) {
    for line in lines {
        println!("\t| {}", line);
    }
}