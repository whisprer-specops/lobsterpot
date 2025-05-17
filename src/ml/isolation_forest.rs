use ndarray::{Array1, Array2, Axis};
use rand::Rng;
use rand::seq::SliceRandom;
use smartcore::linalg::naive::dense_matrix::DenseMatrix;
use smartcore::tree::isolation_forest::IsolationForest;
use std::error::Error;

pub struct IsolationTree {
    split_feature: Option<usize>,
    split_value: f64,
    left: Option<Box<IsolationTree>>,
    right: Option<Box<IsolationTree>>,
    size: usize,
}

impl IsolationTree {
    pub fn new() -> Self {
        IsolationTree {
            split_feature: None,
            split_value: 0.0,
            left: None,
            right: None,
            size: 0,
        }
    }

    pub fn fit(&mut self, data: &Array2<f64>, max_depth: usize, rng: &mut impl Rng) -> Result<(), Box<dyn Error>> {
        self.size = data.nrows();
        if self.size <= 1 || max_depth == 0 {
            return Ok(());
        }

        // Randomly select a feature
        let n_features = data.ncols();
        let split_feature = rng.gen_range(0..n_features);
        self.split_feature = Some(split_feature);

        // Get min and max for the selected feature
        let column = data.column(split_feature);
        let min_val = column.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = column.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if min_val == max_val {
            return Ok(());
        }

        // Random split value between min and max
        self.split_value = rng.gen_range(min_val..max_val);

        // Split data into left and right
        let mut left_data = Vec::new();
        let mut right_data = Vec::new();

        for row in data.rows() {
            if row[split_feature] < self.split_value {
                left_data.push(row.to_vec());
            } else {
                right_data.push(row.to_vec());
            }
        }

        // Convert Vec to Array2
        let left_array = if !left_data.is_empty() {
            Array2::from_shape_vec((left_data.len(), n_features), left_data.into_iter().flatten().collect())?
        } else {
            Array2::zeros((0, n_features))
        };

        let right_array = if !right_data.is_empty() {
            Array2::from_shape_vec((right_data.len(), n_features), right_data.into_iter().flatten().collect())?
        } else {
            Array2::zeros((0, n_features))
        };

        // Recursively build left and right subtrees
        let mut left_tree = Box::new(IsolationTree::new());
        let mut right_tree = Box::new(IsolationTree::new());

        left_tree.fit(&left_array, max_depth - 1, rng)?;
        right_tree.fit(&right_array, max_depth - 1, rng)?;

        self.left = Some(left_tree);
        self.right = Some(right_tree);

        Ok(())
    }

    pub fn path_length(&self, point: &Array1<f64>, depth: usize) -> f64 {
        if self.left.is_none() && self.right.is_none() {
            // Leaf node: return path length adjusted for size
            return depth as f64 + average_path_length(self.size as f64);
        }

        let split_feature = self.split_feature.unwrap();
        let split_value = self.split_value;

        if point[split_feature] < split_value {
            if let Some(left) = &self.left {
                return left.path_length(point, depth + 1);
            }
        } else {
            if let Some(right) = &self.right {
                return right.path_length(point, depth + 1);
            }
        }

        depth as f64
    }
}

pub struct IsolationForestCustom {
    trees: Vec<IsolationTree>,
    n_trees: usize,
    max_depth: usize,
    sample_size: usize,
}

impl IsolationForestCustom {
    pub fn new(n_trees: usize, max_depth: usize, sample_size: usize) -> Self {
        IsolationForestCustom {
            trees: Vec::new(),
            n_trees,
            max_depth,
            sample_size,
        }
    }

    pub fn fit(&mut self, data: &Array2<f64>) -> Result<(), Box<dyn Error>> {
        let mut rng = rand::thread_rng();
        self.trees.clear();

        for _ in 0..self.n_trees {
            // Subsample data
            let mut indices: Vec<usize> = (0..data.nrows()).collect();
            indices.shuffle(&mut rng);
            let sample_indices = indices.into_iter().take(self.sample_size.min(data.nrows())).collect::<Vec<_>>();
            let mut sample_data = Vec::new();

            for idx in sample_indices {
                sample_data.push(data.row(idx).to_vec());
            }

            let sample_array = Array2::from_shape_vec(
                (sample_data.len(), data.ncols()),
                sample_data.into_iter().flatten().collect(),
            )?;

            // Build a tree
            let mut tree = IsolationTree::new();
            tree.fit(&sample_array, self.max_depth, &mut rng)?;
            self.trees.push(tree);
        }

        Ok(())
    }

    pub fn score_samples(&self, data: &Array2<f64>) -> Vec<f64> {
        let mut scores = Vec::new();

        for row in data.rows() {
            let mut total_path_length = 0.0;
            for tree in &self.trees {
                total_path_length += tree.path_length(&row, 0);
            }
            let avg_path_length = total_path_length / self.trees.len() as f64;
            // Anomaly score: higher means more anomalous
            scores.push(2.0_f64.powf(-avg_path_length / average_path_length(self.sample_size as f64)));
        }

        scores
    }
}

// Helper function to calculate average path length for a given sample size
fn average_path_length(n: f64) -> f64 {
    if n <= 1.0 {
        return 0.0;
    }
    2.0 * (n.ln() + std::f64::consts::EULER_GAMMA) - 2.0 * (n - 1.0) / n
}

// Smartcore-based training (for comparison or fallback)
pub async fn train_model(id: i32) -> Result<(), Box<dyn Error>> {
    let x = DenseMatrix::from_2d_array(&[
        &[1.0, 2.0], &[2.0, 3.0], &[3.0, 4.0],
    ]);
    let forest = IsolationForest::fit(&x, Default::default()).unwrap();
    println!("Model {} trained with Smartcore: {:?}", id, forest);
    Ok(())
}

// Custom training for swarms
pub async fn train_model_custom(id: i32, data: &Array2<f64>) -> Result<IsolationForestCustom, Box<dyn Error>> {
    let mut forest = IsolationForestCustom::new(100, 10, 256); // 100 trees, max depth 10, sample size 256
    forest.fit(data)?;
    println!("Custom model {} trained with {} trees", id, forest.n_trees);
    Ok(forest)
}