// src/messaging/mod.rs
pub mod producer;
pub mod consumer;
pub mod config;

// src/messaging/config.rs
use std::time::Duration;
use rdkafka::ClientConfig;

#[derive(Clone)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub group_id: String,
    pub topics: Vec<String>,
    pub client_id: String,
}

impl KafkaConfig {
    pub fn new(
        bootstrap_servers: &str,
        group_id: &str,
        topics: Vec<String>,
        client_id: &str,
    ) -> Self {
        KafkaConfig {
            bootstrap_servers: bootstrap_servers.to_string(),
            group_id: group_id.to_string(),
            topics,
            client_id: client_id.to_string(),
        }
    }

    pub fn to_client_config(&self) -> ClientConfig {
        let mut config = ClientConfig::new();
        config
            .set("bootstrap.servers", &self.bootstrap_servers)
            .set("group.id", &self.group_id)
            .set("client.id", &self.client_id)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .set("socket.timeout.ms", "60000")
            .set("message.timeout.ms", "30000");
        config
    }
}

// src/messaging/messages.rs
use serde::{Serialize, Deserialize};
use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    ThreatFeed,
    ModelUpdate,
    SystemStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreatData {
    pub timestamp: i64,
    pub threat_type: String,
    pub source_ip: Option<IpAddr>,
    pub target_ip: Option<IpAddr>,
    pub confidence: f32,
    pub details: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelUpdate {
    pub model_type: String,
    pub version: String,
    pub weights: Vec<f32>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub instance_id: String,
    pub status: String,
    pub metrics: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KafkaMessage {
    pub message_type: MessageType,
    pub payload: serde_json::Value,
}

// src/messaging/producer.rs
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use super::config::KafkaConfig;
use super::messages::{KafkaMessage, MessageType};
use std::time::Duration;
use anyhow::Result;

pub struct MessageProducer {
    producer: FutureProducer,
    config: KafkaConfig,
}

impl MessageProducer {
    pub fn new(config: KafkaConfig) -> Result<Self> {
        let producer: FutureProducer = config.to_client_config().create()?;
        
        Ok(MessageProducer {
            producer,
            config,
        })
    }

    pub async fn send_message(
        &self,
        topic: &str,
        message: KafkaMessage,
        key: Option<String>,
    ) -> Result<()> {
        let payload = serde_json::to_string(&message)?;
        
        let mut record = FutureRecord::to(topic)
            .payload(&payload);
        
        if let Some(key) = key {
            record = record.key(&key);
        }

        self.producer
            .send(record, Timeout::After(Duration::from_secs(5)))
            .await
            .map_err(|(e, _)| anyhow::anyhow!("Failed to send message: {}", e))?;

        Ok(())
    }

    pub async fn send_threat_data(&self, data: ThreatData) -> Result<()> {
        let message = KafkaMessage {
            message_type: MessageType::ThreatFeed,
            payload: serde_json::to_value(data)?,
        };

        self.send_message("threat_feed", message, None).await
    }

    pub async fn send_model_update(&self, update: ModelUpdate) -> Result<()> {
        let message = KafkaMessage {
            message_type: MessageType::ModelUpdate,
            payload: serde_json::to_value(update)?,
        };

        self.send_message("model_update", message, None).await
    }

    pub async fn send_system_status(&self, status: SystemStatus) -> Result<()> {
        let message = KafkaMessage {
            message_type: MessageType::SystemStatus,
            payload: serde_json::to_value(status)?,
        };

        self.send_message("system_status", message, None).await
    }
}

// src/messaging/consumer.rs
use rdkafka::consumer::{StreamConsumer, Consumer};
use rdkafka::message::{Message, BorrowedMessage};
use super::config::KafkaConfig;
use super::messages::{KafkaMessage, ThreatData, ModelUpdate, SystemStatus};
use std::sync::Arc;
use tokio::sync::mpsc;
use futures::StreamExt;
use anyhow::Result;

pub struct MessageConsumer {
    consumer: StreamConsumer,
    config: KafkaConfig,
}

#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_threat_data(&self, data: ThreatData) -> Result<()>;
    async fn handle_model_update(&self, update: ModelUpdate) -> Result<()>;
    async fn handle_system_status(&self, status: SystemStatus) -> Result<()>;
}

impl MessageConsumer {
    pub fn new(config: KafkaConfig) -> Result<Self> {
        let consumer: StreamConsumer = config.to_client_config().create()?;
        
        Ok(MessageConsumer {
            consumer,
            config,
        })
    }

    pub async fn start(
        &self,
        handler: Arc<dyn MessageHandler>,
        shutdown: mpsc::Receiver<()>,
    ) -> Result<()> {
        self.consumer.subscribe(&self.config.topics)?;

        let mut shutdown_rx = shutdown;
        let mut message_stream = self.consumer.stream();

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("Received shutdown signal");
                    break;
                }
                Some(message) = message_stream.next() => {
                    match message {
                        Ok(borrowed_message) => {
                            if let Err(e) = self.process_message(&borrowed_message, &handler).await {
                                tracing::error!("Error processing message: {}", e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error receiving message: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_message(
        &self,
        borrowed_message: &BorrowedMessage<'_>,
        handler: &Arc<dyn MessageHandler>,
    ) -> Result<()> {
        let payload = borrowed_message
            .payload()
            .ok_or_else(|| anyhow::anyhow!("Empty message payload"))?;

        let message: KafkaMessage = serde_json::from_slice(payload)?;

        match message.message_type {
            MessageType::ThreatFeed => {
                let data: ThreatData = serde_json::from_value(message.payload)?;
                handler.handle_threat_data(data).await?;
            }
            MessageType::ModelUpdate => {
                let update: ModelUpdate = serde_json::from_value(message.payload)?;
                handler.handle_model_update(update).await?;
            }
            MessageType::SystemStatus => {
                let status: SystemStatus = serde_json::from_value(message.payload)?;
                handler.handle_system_status(status).await?;
            }
        }

        Ok(())
    }
}

// Example message handler implementation
pub struct ExampleHandler;

#[async_trait::async_trait]
impl MessageHandler for ExampleHandler {
    async fn handle_threat_data(&self, data: ThreatData) -> Result<()> {
        println!("Received threat data: {:?}", data);
        Ok(())
    }

    async fn handle_model_update(&self, update: ModelUpdate) -> Result<()> {
        println!("Received model update: {:?}", update);
        Ok(())
    }

    async fn handle_system_status(&self, status: SystemStatus) -> Result<()> {
        println!("Received system status: {:?}", status);
        Ok(())
    }
}

// Example usage
pub async fn example_usage() -> Result<()> {
    // Configuration
    let config = KafkaConfig::new(
        "localhost:9092",
        "lobsterpot_group",
        vec!["threat_feed".to_string(), "model_update".to_string(), "system_status".to_string()],
        "lobsterpot_instance_1",
    );

    // Create producer
    let producer = MessageProducer::new(config.clone())?;

    // Create consumer
    let consumer = MessageConsumer::new(config)?;
    let handler = Arc::new(ExampleHandler);
    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

    // Start consumer in a separate task
    let consumer_handle = tokio::spawn(async move {
        consumer.start(handler, shutdown_rx).await
    });

    // Send some example messages
    producer.send_threat_data(ThreatData {
        timestamp: chrono::Utc::now().timestamp(),
        threat_type: "malware".to_string(),
        source_ip: None,
        target_ip: None,
        confidence: 0.95,
        details: "Suspicious activity detected".to_string(),
    }).await?;

    // Wait a bit and then shutdown
    tokio::time::sleep(Duration::from_secs(5)).await;
    shutdown_tx.send(()).await?;
    consumer_handle.await??;

    Ok(())
}