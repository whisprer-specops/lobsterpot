// src/password/mutations.rs
use std::collections::HashMap;

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