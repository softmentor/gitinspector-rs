fn main() {
    let raw = "commit|hash123|Author Name|email@example.com|1600000000|Subject\n10\t5\tfile.rs";
    let mut current_commit: Option<String> = None;
    let mut commit_touches_valid_file = false;

    for line in raw.lines() {
        let line = line.trim();
        println!("Line: {:?}", line);
        if line.starts_with("commit|") {
            let parts: Vec<&str> = line.splitn(6, '|').collect();
            println!("Parts: {:?}", parts);
            if parts.len() == 6 {
                current_commit = Some(parts[1].to_string());
            }
        } else if let Some(ref mut _c) = current_commit {
            let parts: Vec<&str> = line.split_whitespace().collect();
            println!("Numstat parts: {:?}", parts);
            if parts.len() >= 3 {
                commit_touches_valid_file = true;
            }
        }
    }
    println!("Commit touches valid: {}", commit_touches_valid_file);
}
