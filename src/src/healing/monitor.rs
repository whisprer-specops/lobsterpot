// src/healing/monitor.rs
use tokio::time::{Duration, interval};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use super::component::{Component, Healable};
use super::errors::HealingError;

pub struct HealthMonitor {
    components: Arc<Mutex<HashMap<String, Component>>>,
    check_interval: Duration,
}

impl HealthMonitor {
    pub fn new(check_interval: Duration) -> Self {
        HealthMonitor {
            components: Arc::new(Mutex::new(HashMap::new())),
            check_interval,
        }
    }

    pub async fn register_component(&self, component: Component) {
        let mut components = self.components.lock().await;
        components.insert(component.name.clone(), component);
    }

    pub async fn start_monitoring(&self) -> Result<(), HealingError> {
        let components = Arc::clone(&self.components);
        let mut interval = interval(self.check_interval);

        loop {
            interval.tick().await;
            let mut components = components.lock().await;

            for component in components.values_mut() {
                match component.check_health().await {
                    Ok(true) => {
                        println!("Component {} is healthy", component.name);
                    },
                    Ok(false) => {
                        println!("Component {} needs healing", component.name);
                        self.heal_component(component).await?;
                    },
                    Err(e) => {
                        println!("Error checking component {}: {}", component.name, e);
                    }
                }
            }
        }
    }

    async fn heal_component(&self, component: &mut Component) -> Result<(), HealingError> {
        println!("Starting healing process for {}", component.name);

        // First try to isolate the component
        component.isolate().await?;

        // Try to repair
        if let Err(e) = component.repair().await {
            println!("Repair failed: {}. Attempting replacement...", e);
            // If repair fails, try replacement
            component.replace().await?;
        }

        println!("Healing complete for {}", component.name);
        Ok(())
    }
}