// src/ml/ganglion/network.rs
use tch::{nn, Device, Tensor};
use anyhow::Result;

#[derive(Debug)]
pub struct CentralGanglion {
    rnn: RNNModule,
    cnn: CNNModule,
    lstm: LSTMModule,
    dense: nn::Sequential,
    var_store: nn::VarStore,
}

struct RNNModule {
    network: nn::Sequential,
}

struct CNNModule {
    network: nn::Sequential,
}

struct LSTMModule {
    network: nn::Sequential,
}

impl CentralGanglion {
    pub fn new() -> Result<Self> {
        let device = Device::Cpu;
        let mut vs = nn::VarStore::new(device);
        let root = vs.root();

        // RNN for temporal pattern analysis
        let rnn = RNNModule::new(&root.sub("rnn"))?;

        // CNN for spatial pattern analysis
        let cnn = CNNModule::new(&root.sub("cnn"))?;

        // LSTM for long-term memory
        let lstm = LSTMModule::new(&root.sub("lstm"))?;

        // Dense layers for combining outputs
        let dense = nn::seq()
            .add(nn::linear(&root.sub("dense1"), 256, 128, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(&root.sub("dense2"), 128, 64, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(&root.sub("output"), 64, 1, Default::default()))
            .add_fn(|xs| xs.sigmoid());

        Ok(CentralGanglion {
            rnn,
            cnn,
            lstm,
            dense,
            var_store: vs,
        })
    }

    pub fn forward(&self, rnn_input: &Tensor, cnn_input: &Tensor, lstm_input: &Tensor) -> Tensor {
        let rnn_out = self.rnn.forward(rnn_input);
        let cnn_out = self.cnn.forward(cnn_input);
        let lstm_out = self.lstm.forward(lstm_input);

        // Combine outputs
        let combined = Tensor::cat(&[rnn_out, cnn_out, lstm_out], 1);
        self.dense.forward(&combined)
    }

    pub fn save(&self, path: &str) -> Result<()> {
        self.var_store.save(path)?;
        Ok(())
    }

    pub fn load(&mut self, path: &str) -> Result<()> {
        self.var_store.load(path)?;
        Ok(())
    }
}

impl RNNModule {
    fn new(vs: &nn::Path) -> Result<Self> {
        let network = nn::seq()
            .add(nn::rnn_cell(vs, 64, 128, Default::default()))
            .add_fn(|xs| xs.tanh())
            .add(nn::rnn_cell(vs, 128, 64, Default::default()))
            .add_fn(|xs| xs.tanh());

        Ok(RNNModule { network })
    }

    fn forward(&self, input: &Tensor) -> Tensor {
        self.network.forward(input)
    }
}

impl CNNModule {
    fn new(vs: &nn::Path) -> Result<Self> {
        let network = nn::seq()
            .add(nn::conv2d(vs, 1, 32, 3, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::max_pool2d(2, 2, 0, 1, true))
            .add(nn::conv2d(vs, 32, 64, 3, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::max_pool2d(2, 2, 0, 1, true))
            .add_fn(|xs| xs.flat_view());

        Ok(CNNModule { network })
    }

    fn forward(&self, input: &Tensor) -> Tensor {
        self.network.forward(input)
    }
}

impl LSTMModule {
    fn new(vs: &nn::Path) -> Result<Self> {
        let network = nn::seq()
            .add(nn::lstm(vs, 64, 128, Default::default()))
            .add_fn(|xs| xs.tanh())
            .add(nn::lstm(vs, 128, 64, Default::default()))
            .add_fn(|xs| xs.tanh());

        Ok(LSTMModule { network })
    }

    fn forward(&self, input: &Tensor) -> Tensor {
        self.network.forward(input)
    }
}
