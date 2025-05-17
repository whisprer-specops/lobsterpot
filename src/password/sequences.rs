// src/password/sequences.rs
use chrono::Datelike;

pub fn generate_year_suffixes() -> Vec<String> {
    let current_year = chrono::Local::now().year();
    let mut years = Vec::new();
    
    for year in 1940..=current_year {
        years.push(format!("{}", year % 100));
    }
    
    // Add common number patterns
    years.extend(vec!["00", "01", "69", "123", "1234"].iter().map(|&s| s.to_string()));
    years
}

pub fn generate_fibonacci(n: usize) -> Vec<String> {
    let mut sequence = Vec::with_capacity(n);
    if n >= 1 { sequence.push(0); }
    if n >= 2 { sequence.push(1); }
    
    while sequence.len() < n {
        let next = sequence[sequence.len() - 1] + sequence[sequence.len() - 2];
        sequence.push(next);
    }
    
    sequence.iter().map(|n| n.to_string()).collect()
}

pub fn generate_lucas(n: usize) -> Vec<String> {
    let mut sequence = Vec::with_capacity(n);
    if n >= 1 { sequence.push(2); }
    if n >= 2 { sequence.push(1); }
    
    while sequence.len() < n {
        let next = sequence[sequence.len() - 1] + sequence[sequence.len() - 2];
        sequence.push(next);
    }
    
    sequence.iter().map(|n| n.to_string()).collect()
}