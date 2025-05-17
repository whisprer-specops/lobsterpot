// src/password/cracker.rs
use rayon::prelude::*;
use sha2::{Sha256, Digest};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use super::mutations::{CharacterMap, mutate_word, case_mutations};
use super::sequences::{generate_fibonacci, generate_lucas, generate_year_suffixes};

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