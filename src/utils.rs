pub fn print_in_box(lines: &[&str]) {
    // Print each line, padded and enclosed
    for line in lines {
        println!("\t| {}", line);
    }
}