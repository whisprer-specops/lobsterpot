// src/ml/isolation_forest.rs
use ndarray::{Array1, Array2, Axis};
use rand::Rng;
use rand::seq::SliceRandom;

pub struct IsolationTree {
    split_feature: Option<usize>,
    split_value: f64,
    left: Option<Box<IsolationTree>>,
    right: Option<Box<IsolationTree>>,
    size: usize,
}

// ... rest of your existing isolation forest code ...