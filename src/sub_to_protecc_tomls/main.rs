mod database;
mod healing;
mod logging;
mod ml;
mod network;
mod password;
mod threat;
mod coordination;

use rayon::prelude::*;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("LobsterPot starting with swarm capability...");

let coord_task = coordination::mesh::coordinate_swarm(i);

    // Swarm tasks for accelerated learning
    let tasks: Vec<_> = (0..10)
        .map(|i| {
            let ml_task = ml::isolation_forest::train_model(i);
            let net_task = network::monitor::test_proxy("http://proxy.example.com:8080".to_string());
            (ml_task, net_task)
        })
        .collect();

    tasks.into_par_iter().for_each(|(ml_task, net_task)| {
        tokio::spawn(async move {
            if let Ok(ml_result) = ml_task.await {
                println!("ML task {} trained: {:?}", ml_task, ml_result);
            }
            if let Ok(net_result) = net_task.await {
                println!("Network task {}: {:?}", net_task, net_result);
            }
        });
    });

    Ok(())
}