use std::fs;
use regex::Regex;

/// Simplified Cyclomatic Complexity Metric Calculation
#[derive(Debug, Default, Clone)]
pub struct FileMetrics {
    pub complexity: u32,
}

pub fn calculate_file_complexity(file_path: &str) -> Option<FileMetrics> {
    let content = fs::read_to_string(file_path).ok()?;
    
    // Improved, simplified regex to count common branching keywords across most C-style/Python languages
    // Features parity with basic cyclomatic complexity.
    let re = Regex::new(r"\b(if|else|while|for|catch|case|switch|&&|\|\||\?)\b").ok()?;
    
    // Base complexity is 1, add 1 for every branching point found
    let complexity = 1 + re.find_iter(&content).count() as u32;

    Some(FileMetrics { complexity })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_calculate_file_complexity() {
        let test_file = "test_metrics.rs";
        let content = "fn main() {\n if true { println!(\"true\"); }\n while false {} \n}";
        fs::write(test_file, content).unwrap();

        let metrics = calculate_file_complexity(test_file).unwrap();
        // Base 1 + "if" + "while" = 3
        assert_eq!(metrics.complexity, 3);

        fs::remove_file(test_file).unwrap();
    }
}
