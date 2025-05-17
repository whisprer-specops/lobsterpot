// src/ml/ganglion/trainer.rs
use tch::{Tensor, Device};
use super::network::CentralGanglion;
use anyhow::Result;

pub struct GanglionTrainer {
    ganglion: CentralGanglion,
    optimizer: tch::nn::Optimizer,
    cellular_automaton: CellularAutomaton,
}

impl GanglionTrainer {
    pub fn new(ganglion: CentralGanglion, learning_rate: f64) -> Result<Self> {
        let optimizer = tch::nn::Adam::default().build(&ganglion.var_store, learning_rate)?;
        let cellular_automaton = CellularAutomaton::new(10);

        Ok(GanglionTrainer {
            ganglion,
            optimizer,
            cellular_automaton,
        })
    }

    pub fn train_step(
        &mut self,
        rnn_input: &Tensor,
        cnn_input: &Tensor,
        lstm_input: &Tensor,
        target: &Tensor,
    ) -> Result<f64> {
        self.optimizer.zero_grad();

        // Get cellular automaton influence
        self.cellular_automaton.update();
        let ca_influence = self.cellular_automaton.get_influence();

        // Forward pass with cellular automaton influence
        let output = self.ganglion.forward(rnn_input, cnn_input, lstm_input);
        let output = output * (1.0 + ca_influence as f64);

        // Calculate loss
        let loss = output.binary_cross_entropy(target, None);
        
        // Backward pass
        loss.backward();
        
        // Update weights
        self.optimizer.step();

        Ok(f64::from(loss))
    }

    pub fn predict(
        &self,
        rnn_input: &Tensor,
        cnn_input: &Tensor,
        lstm_input: &Tensor,
    ) -> Tensor {
        let ca_influence = self.cellular_automaton.get_influence();
        let output = self.ganglion.forward(rnn_input, cnn_input, lstm_input);
        output * (1.0 + ca_influence as f64)
    }
}
