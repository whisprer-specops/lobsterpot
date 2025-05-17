mod database;
mod healing;
mod logging;
mod ml;
mod network;
mod password;
mod threat;
mod coordination;

use ndarray::{Array2, array};
use rayon::prelude::*;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("LobsterPot starting with swarm capability...");

    // Sample data for training (replace with real data from network/threat modules)
    let data = array![
        [1.0, 2.0], [2.0, 3.0], [3.0, 4.0],
        [10.0, 10.0] // Anomaly
    ];

    // Swarm tasks for accelerated learning
    let tasks: Vec<_> = (0..10)
        .map(|i| {
            let ml_task = ml::isolation_forest::train_model_custom(i, &data);
            let net_task = network::monitor::test_proxy("http://proxy.example.com:8080".to_string());
            (ml_task, net_task)
        })
        .collect();

    tasks.into_par_iter().for_each(|(ml_task, net_task)| {
        tokio::spawn(async move {
            match ml_task.await {
                Ok(forest) => {
                    // Score some data
                    let scores = forest.score_samples(&data);
                    println!("Swarm task {} scores: {:?}", forest, scores);
                }
                Err(e) => eprintln!("ML task error: {}", e),
            }
            if let Ok(net_result) = net_task.await {
                println!("Network task: {:?}", net_result);
            }
        });
    });

    Ok(())
}