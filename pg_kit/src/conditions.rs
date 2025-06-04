use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

use tokio::time::sleep;

use autonomic_operation::operation::OperationParameters;
use autonomic_operation::sensor::ActivationCondition;

#[derive(Clone, Debug)]
pub struct Interval {
    interval: u32, // interval in seconds
    parameters: Option<Arc<dyn OperationParameters>>,
}

impl Interval {
    /// Creates a new interval rule.
    ///
    /// # Arguments
    /// - `interval` - The interval in `seconds`.
    /// - `parameters` - Optional parameters to be returned when the condition is met.
    ///
    /// # Note
    /// `interval` must be greater than `0`. If `interval` is `0`, it will be set to `1`.
    pub fn new(interval: u32, parameters: Option<Arc<dyn OperationParameters>>) -> Self {
        Interval {
            interval: interval.max(1),
            parameters,
        }
    }
}

#[async_trait]
impl ActivationCondition for Interval {
    async fn activate(&self) -> Option<Arc<dyn OperationParameters>> {
        let interval = self.interval;
        sleep(Duration::from_secs(interval as u64)).await;
        tracing::info!("Interval condition activated");
        self.parameters.clone()
    }
}
