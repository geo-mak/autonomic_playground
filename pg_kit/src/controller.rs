use std::path::Path;
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use autonomic_events::trace_error;
use autonomic_events::trace_info;
use autonomic_events::trace_warn;

use autonomic_controllers::controller::{ControlContext, Controller, ControllerResult};

pub struct PGController {
    id: &'static str,
    description: &'static str,
    store: &'static str,
    default: &'static str,
    poll: Duration,
}

impl PGController {
    pub fn new(
        id: &'static str,
        description: &'static str,
        store: &'static str,
        default_state: &'static str,
        poll: Duration,
    ) -> Self {
        let data_path = Path::new(store);
        if !data_path.exists() {
            std::fs::write(data_path, default_state).ok();
        }

        Self {
            id,
            description,
            store,
            default: default_state,
            poll,
        }
    }
}

#[async_trait]
impl Controller for PGController {
    fn id(&self) -> &'static str {
        self.id
    }

    fn description(&self) -> &'static str {
        self.description
    }

    async fn notified(&self) {
        loop {
            tokio::time::sleep(self.poll).await;
            let content = match tokio::fs::read_to_string(self.store).await {
                Ok(c) => c,
                Err(e) => {
                    trace_warn!(
                        source = self.id,
                        message = format!("Failed to read file: {e}")
                    );
                    continue;
                }
            };
            if content != self.default {
                trace_warn!(source = self.id, message = "State change has been detected");
                return;
            }
        }
    }

    async fn perform(&self, _: &ControlContext) -> ControllerResult {
        trace_info!(source = self.id, message = "Starting the control operation");
        match tokio::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.store)
            .await
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(self.default.as_bytes()).await {
                    return ControllerResult::ErrMsg(e.to_string().into());
                }
                trace_info!(source = self.id, message = "The state has been corrected");
                ControllerResult::Ok
            }
            Err(e) => {
                trace_error!(
                    source = self.id,
                    message = "The correction action has failed"
                );
                ControllerResult::ErrMsg(e.to_string().into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autonomic_controllers::controller::{ControlContext, ControllerResult};
    use std::fs;

    static CTRL_NAME: &str = "test_ctr";
    static STORE: &str = "store";
    static DEFAULT_STATE: &str = "default state";

    fn cleanup_data_file(store: &str) {
        let _ = fs::remove_file(store);
    }

    #[tokio::test]
    async fn test_ctrl_perform_writes_default() {
        cleanup_data_file(STORE);
        let controller = PGController::new(
            CTRL_NAME,
            CTRL_NAME,
            STORE,
            DEFAULT_STATE,
            Duration::from_secs(1),
        );
        let ctx = ControlContext::new();
        let result = controller.perform(&ctx).await;
        assert!(matches!(result, ControllerResult::Ok));
        let content = fs::read_to_string(STORE).unwrap();
        assert_eq!(content, DEFAULT_STATE);
        cleanup_data_file(STORE);
    }

    #[tokio::test]
    async fn test_pgcontroller_notify_receives_event() {
        cleanup_data_file(STORE);
        let controller = PGController::new(
            CTRL_NAME,
            CTRL_NAME,
            STORE,
            DEFAULT_STATE,
            Duration::from_secs(1),
        );
        fs::write(STORE, "new state").unwrap();
        let notify_fut = controller.notified();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(2), notify_fut).await;
        cleanup_data_file(STORE);
    }
}
