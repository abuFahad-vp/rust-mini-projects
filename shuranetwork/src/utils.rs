use std::io::Write;

pub fn get_value(prompt: &str) -> String {
    print!("{prompt}");
    let mut value = String::new();
    std::io::stdout().flush().expect("Failed to flush the stdout.");
    std::io::stdin().read_line(&mut value).expect("Failed to read the miner address");
    value.trim().to_string()
}