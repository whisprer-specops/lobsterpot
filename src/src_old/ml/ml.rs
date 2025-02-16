// src/ml/mod.rs
pub mod isolation_forest;

// src/ml/isolation_forest.rs
use ndarray::{Array1, Array2, Axis};
use rand::Rng;
use rand::seq::SliceRandom;
use std::cmp;

pub struct IsolationTree {
    split_feature: Option<usize>,
    split_value: f64,
    left: Option<Box<IsolationTree>>,
    right: Option<Box<IsolationTree>>,
    size: usize,
}

pub struct IsolationForest {
    trees: Vec<IsolationTree>,
    n_estimators: usize,
    max_samples: usize,
    contamination: f64,
}

impl IsolationTree {
    pub fn new(data: &Array2<f64>, height_limit: i32, current_height: i32) -> Self {
        let n_samples = data.nrows();
        
        // Base cases
        if current_height >= height_limit || n_samples <= 1 {
            return IsolationTree {
                split_feature: None,
                split_value: 0.0,
                left: None,
                right: None,
                size: n_samples,
            };
        }

        let mut rng = rand::thread_rng();
        let n_features = data.ncols();
        
        // Randomly select feature
        let split_feature = rng.gen_range(0..n_features);
        
        // Get min and max values for the selected feature
        let col = data.column(split_feature);
        let min_val = col.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = col.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        // Generate split value
        let split_value = rng.gen_range(min_val..=max_val);
        
        // Split data
        let (left_data, right_data): (Vec<_>, Vec<_>) = data.outer_iter()
            .partition(|row| row[split_feature] < split_value);
        
        // Convert back to Array2
        let left_data = Array2::from_shape_vec(
            (left_data.len(), n_features),
            left_data.into_iter().flat_map(|row| row.to_vec()).collect(),
        ).unwrap_or(Array2::zeros((0, n_features)));
        
        let right_data = Array2::from_shape_vec(
            (right_data.len(), n_features),
            right_data.into_iter().flat_map(|row| row.to_vec()).collect(),
        ).unwrap_or(Array2::zeros((0, n_features)));

        IsolationTree {
            split_feature: Some(split_feature),
            split_value,
            left: Some(Box::new(IsolationTree::new(&left_data, height_limit, current_height + 1))),
            right: Some(Box::new(IsolationTree::new(&right_data, height_limit, current_height + 1))),
            size: n_samples,
        }
    }

    pub fn path_length(&self, x: &Array1<f64>, current_height: f64) -> f64 {
        if self.left.is_none() || self.right.is_none() {
            return current_height + c(self.size);
        }

        if let Some(split_feature) = self.split_feature {
            if x[split_feature] < self.split_value {
                self.left.as_ref().unwrap().path_length(x, current_height + 1.0)
            } else {
                self.right.as_ref().unwrap().path_length(x, current_height + 1.0)
            }
        } else {
            current_height + c(self.size)
        }
    }
}

impl IsolationForest {
    pub fn new(n_estimators: usize, max_samples: usize, contamination: f64) -> Self {
        IsolationForest {
            trees: Vec::new(),
            n_estimators,
            max_samples,
            contamination,
        }
    }

    pub fn fit(&mut self, data: &Array2<f64>) {
        let height_limit = (self.max_samples as f64).log2().ceil() as i32;
        
        for _ in 0..self.n_estimators {
            // Randomly sample data
            let mut indices: Vec<usize> = (0..data.nrows()).collect();
            indices.shuffle(&mut rand::thread_rng());
            let sample_size = cmp::min(self.max_samples, data.nrows());
            let sample_indices = &indices[..sample_size];
            
            let sample_data = Array2::from_shape_vec(
                (sample_size, data.ncols()),
                sample_indices.iter()
                    .flat_map(|&i| data.row(i).to_vec())
                    .collect()
            ).unwrap();
            
            // Create and add tree
            let tree = IsolationTree::new(&sample_data, height_limit, 0);
            self.trees.push(tree);
        }
    }

    pub fn predict(&self, data: &Array2<f64>) -> Vec<bool> {
        let scores = self.decision_function(data);
        let threshold = self.calculate_threshold(&scores);
        
        scores.iter()
            .map(|&score| score < threshold)
            .collect()
    }

    pub fn decision_function(&self, data: &Array2<f64>) -> Vec<f64> {
        let mut scores = Vec::with_capacity(data.nrows());
        
        for i in 0..data.nrows() {
            let x = data.row(i).to_owned();
            let avg_path_length: f64 = self.trees.iter()
                .map(|tree| tree.path_length(&x, 0.0))
                .sum::<f64>() / self.trees.len() as f64;
            
            let score = 2_f64.powf(-avg_path_length / c(self.max_samples));
            scores.push(score);
        }
        
        scores
    }

    fn calculate_threshold(&self, scores: &[f64]) -> f64 {
        let mut sorted_scores = scores.to_vec();
        sorted_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let threshold_index = ((scores.len() as f64) * self.contamination) as usize;
        sorted_scores[threshold_index]
    }
}

// Helper function c(n) as defined in the Isolation Forest paper
fn c(n: usize) -> f64 {
    if n <= 1 {
        return 0.0;
    }
    let n = n as f64;
    2.0 * (n - 1.0).ln() + 0.5772156649
}

// Example usage
pub fn example_usage() {
    // Create sample data
    let data = Array2::from_shape_vec(
        (5, 2),
        vec![
            1.0, 2.0,
            2.0, 3.0,
            3.0, 4.0,
            10.0, 10.0,  // Anomaly
            2.5, 3.5,
        ]
    ).unwrap();

    // Create and train the model
    let mut forest = IsolationForest::new(100, 256, 0.1);
    forest.fit(&data);

    // Make predictions
    let predictions = forest.predict(&data);
    println!("Predictions (true means anomaly): {:?}", predictions);
}