// src/password/mutations.rs
use std::collections::HashMap;

use super::rainbow::RainbowTable;


pub struct CharacterMap {
    mappings: HashMap<char, Vec<char>>
}

impl CharacterMap {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();
        mappings.insert('a', vec!['@', '4']);
        mappings.insert('e', vec!['3']);
        mappings.insert('i', vec!['1', '!']);
        mappings.insert('o', vec!['0']);
        mappings.insert('s', vec!['$', '5']);
        CharacterMap { mappings }
    }

    pub fn get_mutations(&self, c: char) -> Vec<char> {
        self.mappings.get(&c)
            .map(|chars| chars.clone())
            .unwrap_or_else(|| vec![c])
    }
}

pub fn mutate_word(word: &str, char_map: &CharacterMap) -> Vec<String> {
    let mut results = Vec::new();
    
    fn generate_mutations(
        current: String,
        remaining: &[char],
        char_map: &CharacterMap,
        results: &mut Vec<String>
    ) {
        if remaining.is_empty() {
            results.push(current);
            return;
        }

        let curr_char = remaining[0];
        let mutations = char_map.get_mutations(curr_char);
        
        for mutation in mutations {
            let mut new_current = current.clone();
            new_current.push(mutation);
            generate_mutations(new_current, &remaining[1..], char_map, results);
        }
    }

    generate_mutations(String::new(), &word.chars().collect::<Vec<_>>(), char_map, &mut results);
    results
}

pub fn case_mutations(word: &str) -> Vec<String> {
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    let combinations = 2u32.pow(n as u32);
    let mut results = Vec::with_capacity(combinations as usize);

    for i in 0..combinations {
        let mut new_word = String::with_capacity(n);
        for (j, &c) in chars.iter().enumerate() {
            if (i & (1 << j)) == 0 {
                new_word.push(c.to_ascii_lowercase());
            } else {
                new_word.push(c.to_ascii_uppercase());
            }
        }
        results.push(new_word);
    }
    results
}

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

// src/password/cracker.rs
use rayon::prelude::*;
use sha2::{Sha256, Digest};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct PasswordCracker {
    char_map: CharacterMap,
    common_passwords: Vec<String>,
    found: Arc<AtomicBool>,
}

impl PasswordCracker {
    pub fn new() -> Self {
        let common_passwords = vec![
            "password", "123456", "qwerty", "admin", "letmein",
            "welcome", "monkey", "dragon", "baseball", "football"
        ].into_iter().map(String::from).collect();

        PasswordCracker {
            char_map: CharacterMap::new(),
            common_passwords,
            found: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn crack_password(&self, hash_to_match: &str) -> Option<String> {
        let start_time = Instant::now();
        println!("Starting password cracking attempt...");

        // Try common passwords first
        let result = self.try_common_passwords(hash_to_match);
        if result.is_some() {
            return result;
        }

        // Try mutations of common passwords
        let result = self.try_password_mutations(hash_to_match);
        if result.is_some() {
            return result;
        }

        // Try sequences
        let result = self.try_sequences(hash_to_match);
        if result.is_some() {
            return result;
        }

        println!("Password not found. Time elapsed: {:?}", start_time.elapsed());
        None
    }

    fn try_common_passwords(&self, hash_to_match: &str) -> Option<String> {
        self.common_passwords.par_iter()
            .find_map_any(|password| {
                if self.check_password(password, hash_to_match) {
                    Some(password.clone())
                } else {
                    None
                }
            })
    }

    fn try_password_mutations(&self, hash_to_match: &str) -> Option<String> {
        self.common_passwords.par_iter()
            .flat_map(|password| {
                let mutations = mutate_word(password, &self.char_map);
                let case_mutations: Vec<String> = mutations.into_iter()
                    .flat_map(|m| case_mutations(&m))
                    .collect();
                case_mutations
            })
            .find_map_any(|mutation| {
                if self.check_password(&mutation, hash_to_match) {
                    Some(mutation)
                } else {
                    None
                }
            })
    }

    fn try_sequences(&self, hash_to_match: &str) -> Option<String> {
        let sequences: Vec<String> = generate_fibonacci(20)
            .into_iter()
            .chain(generate_lucas(20))
            .chain(generate_year_suffixes())
            .collect();

        sequences.par_iter()
            .find_map_any(|seq| {
                if self.check_password(seq, hash_to_match) {
                    Some(seq.clone())
                } else {
                    None
                }
            })
    }

    fn check_password(&self, password: &str, hash_to_match: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let result = format!("{:x}", hasher.finalize());
        result == hash_to_match
    }
}

// Example usage
pub fn example_usage() {
    // Create a hash of a password to crack (in real usage, this would be the hash you're trying to crack)
    let mut hasher = Sha256::new();
    hasher.update(b"password123");
    let hash_to_crack = format!("{:x}", hasher.finalize());

    let cracker = PasswordCracker::new();
    match cracker.crack_password(&hash_to_crack) {
        Some(password) => println!("Password found: {}", password),
        None => println!("Password not found"),
    }
}