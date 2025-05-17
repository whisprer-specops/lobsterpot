// src/password/rainbow.rs
use std::fs::OpenOptions;
use std::path::Path;
use memmap2::MmapOptions;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{BufWriter, Write};
use rayon::prelude::*;

const CHAIN_LENGTH: usize = 1000;
const MIN_PASSWORD_LENGTH: usize = 6;
const MAX_PASSWORD_LENGTH: usize = 10;

#[derive(Serialize, Deserialize, Debug)]
pub struct RainbowChain {
    start: String,
    end: String,
}

#[derive(Serialize, Deserialize)]
pub struct RainbowTable {
    chains: Vec<RainbowChain>,
    charset: Vec<char>,
}

impl RainbowTable {
    pub fn new(charset: Vec<char>) -> Self {
        RainbowTable {
            chains: Vec::new(),
            charset,
        }
    }

    pub fn generate(&mut self, num_chains: usize) {
        let pb = ProgressBar::new(num_chains as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} chains").unwrap());

        let chains: Vec<RainbowChain> = (0..num_chains)
            .into_par_iter()
            .map(|_| {
                let start = self.generate_random_password();
                let end = self.generate_chain(&start);
                RainbowChain { start, end }
            })
            .collect();

        pb.finish_with_message("Table generation complete");
        self.chains = chains;
    }

    fn generate_chain(&self, start: &str) -> String {
        let mut current = start.to_string();
        
        for i in 0..CHAIN_LENGTH {
            let hash = self.hash(&current);
            current = self.reduce(&hash, i);
        }
        
        current
    }

    fn hash(&self, input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn reduce(&self, hash: &str, position: usize) -> String {
        let charset_len = self.charset.len();
        let mut result = String::with_capacity(8);
        
        for (i, byte) in hash.as_bytes().iter().enumerate().take(8) {
            let index = (((*byte as usize) + position) % charset_len);
            result.push(self.charset[index]);
        }
        
        result
    }

    fn generate_random_password(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let len = rng.gen_range(MIN_PASSWORD_LENGTH..=MAX_PASSWORD_LENGTH);
        
        (0..len)
            .map(|_| {
                let idx = rng.gen_range(0..self.charset.len());
                self.charset[idx]
            })
            .collect()
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .open(path)?;
        
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        
        bincode::deserialize(&mmap[..])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    pub fn crack_password(&self, hash_to_crack: &str) -> Option<String> {
        println!("Attempting to crack hash: {}", hash_to_crack);
        let pb = ProgressBar::new(CHAIN_LENGTH as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} steps").unwrap());

        // Try each possible reduction
        for i in (0..CHAIN_LENGTH).rev() {
            pb.inc(1);
            let mut current = hash_to_crack.to_string();
            
            // Follow the chain from the hash to the end
            for j in i..CHAIN_LENGTH {
                current = self.reduce(&current, j);
                if j < CHAIN_LENGTH - 1 {
                    current = self.hash(&current);
                }
            }

            // Look for matching chain
            if let Some(chain) = self.chains.iter().find(|c| c.end == current) {
                // Regenerate the chain to find the actual password
                if let Some(password) = self.regenerate_chain_and_find_password(
                    &chain.start, 
                    hash_to_crack
                ) {
                    pb.finish_with_message("Password found!");
                    return Some(password);
                }
            }
        }

        pb.finish_with_message("Password not found");
        None
    }

    fn regenerate_chain_and_find_password(
        &self,
        start: &str,
        target_hash: &str
    ) -> Option<String> {
        let mut current = start.to_string();
        
        for i in 0..CHAIN_LENGTH {
            let hash = self.hash(&current);
            if hash == target_hash {
                return Some(current);
            }
            current = self.reduce(&hash, i);
        }
        
        None
    }
}

// Example usage
pub fn example_usage() {
    // Define character set
    let charset: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();

    // Create and generate rainbow table
    let mut table = RainbowTable::new(charset);
    println!("Generating rainbow table...");
    table.generate(10000); // Generate 10,000 chains

    // Save the table
    println!("Saving rainbow table...");
    table.save_to_file("rainbow_table.bin").unwrap();

    // Load the table
    println!("Loading rainbow table...");
    let loaded_table = RainbowTable::load_from_file("rainbow_table.bin").unwrap();

    // Try to crack a password
    let password_to_crack = "password123";
    let mut hasher = Sha256::new();
    hasher.update(password_to_crack.as_bytes());
    let hash_to_crack = format!("{:x}", hasher.finalize());

    println!("Attempting to crack password...");
    match loaded_table.crack_password(&hash_to_crack) {
        Some(password) => println!("Password found: {}", password),
        None => println!("Password not found in rainbow table"),
    }
}

impl PasswordCracker {
    // Add this method to your existing PasswordCracker implementation
    pub fn crack_with_rainbow_table(&self, hash_to_match: &str, table_path: &str) -> Option<String> {
        println!("Attempting to crack password using rainbow table...");
        
        match RainbowTable::load_from_file(table_path) {
            Ok(table) => {
                match table.crack_password(hash_to_match) {
                    Some(password) => {
                        println!("Password found in rainbow table!");
                        Some(password)
                    },
                    None => {
                        println!("Password not found in rainbow table, falling back to regular methods...");
                        self.crack_password(hash_to_match)
                    }
                }
            },
            Err(e) => {
                println!("Failed to load rainbow table: {}. Falling back to regular methods...", e);
                self.crack_password(hash_to_match)
            }
        }
    }
}