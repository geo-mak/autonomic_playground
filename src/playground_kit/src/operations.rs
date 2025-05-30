use std::any::Any;
use std::time::Duration;

use async_trait::async_trait;
use autonomic_core::operation::{Operation, OperationParameters, OperationResult};
use autonomic_core::traits::{Describe, Identity};
use autonomic_core::trace_info;
use serde::{Deserialize, Serialize};

/// Retry parameters for the `PlaygroundOperation`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Retry {
    pub retries: u8,
    pub delay_ms: u32,
}

impl Retry {
    pub fn new(retries: u8, delay_ms: u32) -> Self {
        Self { retries, delay_ms }
    }
}

impl OperationParameters for Retry {
    fn as_parameters(&self) -> &dyn Any {
        self
    }
}

/// Enum to represent the possible outcomes when activating a `PlaygroundOperation`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Play {
    NormalResult(OperationResult),
    Panic,
}

/// Parameters for the `PlaygroundOperation`.
/// It contains the `Play` result, the number of seconds to sleep before returning the result, and an optional `Retry` parameters.
/// `Retry` parameters are used to retry the operation in case the result is an error exclusively.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayParameters {
    result: Play,
    sleep_sec: u8,
    retry: Option<Retry>,
}

impl PlayParameters {
    pub fn new(result: Play, retry: Option<Retry>, sleep_sec: u8) -> Self {
        Self {
            result,
            sleep_sec,
            retry,
        }
    }
}

impl OperationParameters for PlayParameters {
    fn as_parameters(&self) -> &dyn Any {
        self
    }
}

/// Operation to experiment with in the playground.
/// This operation is a real operation that executes within the system in the same way as any other operation.
/// It takes a `PlayParameters` as input and performs according to the parameters.
pub struct PlaygroundOperation {
    id: &'static str,
    description: &'static str,
}

impl PlaygroundOperation {
    pub fn new(id: &'static str, description: &'static str) -> Self {
        Self { id, description }
    }

    async fn run(sleep_sec: u8, result: &Play) -> OperationResult {
        if sleep_sec != 0 {
            tokio::time::sleep(Duration::from_secs(sleep_sec as u64)).await;
        }
        match result {
            Play::NormalResult(op_result) => op_result.clone(),
            Play::Panic => panic!("Unexpected Error in Playground Operation"),
        }
    }

    async fn retry_run(&self, play: &PlayParameters) -> OperationResult {
        let mut result = Self::run(play.sleep_sec, &play.result).await;
        if result.is_err() {
            if let Some(retry_params) = &play.retry {
                if retry_params.retries > 0 {
                    for attempt in 1..=retry_params.retries {
                        tokio::time::sleep(Duration::from_millis(retry_params.delay_ms as u64))
                            .await;
                        trace_info!(
                            source = self.id,
                            message = format!("Attempt={} to perform", attempt)
                        );
                        result = Self::run(play.sleep_sec, &play.result).await;
                        if result.is_ok() {
                            break;
                        }
                    }
                }
            }
        }
        result
    }
}

impl Identity for PlaygroundOperation {
    type ID = &'static str;
    fn id(&self) -> Self::ID {
        self.id
    }
}

impl Describe for PlaygroundOperation {
    type Description = &'static str;
    fn describe(&self) -> Self::Description {
        self.description
    }
}

#[async_trait]
impl Operation for PlaygroundOperation {
    async fn perform(&self, parameters: Option<&dyn OperationParameters>) -> OperationResult {
        if let Some(params) = parameters {
            if let Some(play) = params.as_parameters().downcast_ref::<PlayParameters>() {
                self.retry_run(play).await
            } else {
                OperationResult::err_str("Unexpected parameters")
            }
        } else {
            OperationResult::err_str("Parameters required")
        }
    }
}
