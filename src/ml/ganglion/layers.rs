// Example usage
pub fn example_usage() -> Result<()> {
    // Initialize the Central Ganglion
    let ganglion = CentralGanglion::new()?;
    let mut trainer = GanglionTrainer::new(ganglion, 0.001)?;

    // Create example inputs
    let rnn_input = Tensor::rand(&[32, 64], (tch::Kind::Float, Device::Cpu));
    let cnn_input = Tensor::rand(&[32, 1, 28, 28], (tch::Kind::Float, Device::Cpu));
    let lstm_input = Tensor::rand(&[32, 64], (tch::Kind::Float, Device::Cpu));
    let target = Tensor::rand(&[32, 1], (tch::Kind::Float, Device::Cpu));

    // Training loop
    for epoch in 0..10 {
        let loss = trainer.train_step(&rnn_input, &cnn_input, &lstm_input, &target)?;
        println!("Epoch {}, Loss: {}", epoch, loss);
    }

    // Make predictions
    let predictions = trainer.predict(&rnn_input, &cnn_input, &lstm_input);
    println!("Predictions shape: {:?}", predictions.size());

    Ok(())
}