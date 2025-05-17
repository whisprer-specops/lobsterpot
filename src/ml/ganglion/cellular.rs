// src/ml/ganglion/cellular.rs
use ndarray::{Array2, ArrayView2};
use rand::Rng;

pub struct CellularAutomaton {
    grid: Array2<f32>,
    size: usize,
}

impl CellularAutomaton {
    pub fn new(size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let grid = Array2::from_shape_fn((size, size), |_| rng.gen::<f32>());
        
        CellularAutomaton { grid, size }
    }

    pub fn update(&mut self) {
        let old_grid = self.grid.clone();
        
        for i in 1..self.size-1 {
            for j in 1..self.size-1 {
                let neighbors = self.get_neighbors(&old_grid.view(), i, j);
                self.grid[[i, j]] = self.apply_rules(&neighbors);
            }
        }
    }

    fn get_neighbors(&self, grid: &ArrayView2<f32>, i: usize, j: usize) -> Vec<f32> {
        let mut neighbors = Vec::with_capacity(8);
        
        for di in -1..=1 {
            for dj in -1..=1 {
                if di == 0 && dj == 0 { continue; }
                let ni = (i as i32 + di) as usize;
                let nj = (j as i32 + dj) as usize;
                neighbors.push(grid[[ni, nj]]);
            }
        }
        
        neighbors
    }

    fn apply_rules(&self, neighbors: &[f32]) -> f32 {
        let sum: f32 = neighbors.iter().sum();
        let avg = sum / neighbors.len() as f32;
        
        // Complex cellular automaton rules
        if avg > 0.5 {
            (avg * 1.5).min(1.0)
        } else {
            (avg * 0.5).max(0.0)
        }
    }

    pub fn get_influence(&self) -> f32 {
        self.grid.mean().unwrap_or(0.0)
    }
}
