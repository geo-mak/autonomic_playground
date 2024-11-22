use autonomic::core::operation::OpState;
use autonomic::core::serde::AnySerializable;
use autonomic::core::service::ControllerClient;
use autonomic::openapi::client::OpenAPIClient;
use lazy_static::lazy_static;
use reqwest::Client;
use tokio_stream::StreamExt;

lazy_static! {
    pub static ref AUTONOMIC_CLIENT: OpenAPIClient<'static> =
        OpenAPIClient::new(Client::new(), "http://127.0.0.1:8000");
}

pub struct PlaygroundOpenAPIClient;

#[allow(dead_code)]
impl PlaygroundOpenAPIClient {
    pub async fn get_operation(controller_id: &str, operation_id: &str) {
        let result = AUTONOMIC_CLIENT
            .operation(controller_id, operation_id)
            .await;
        match result {
            Ok(op_info) => {
                tracing::info!(
                    "Received info about operation {}: {:?}",
                    operation_id,
                    op_info
                );
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn get_all_operations(controller_id: &str) {
        let result = AUTONOMIC_CLIENT.operations(controller_id).await;
        match result {
            Ok(ops_infos) => {
                tracing::info!("Received current operations: {:?}", ops_infos);
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn get_active_operations(controller_id: &str) {
        let result = AUTONOMIC_CLIENT.active_operations(controller_id).await;
        match result {
            Ok(ops) => {
                tracing::info!("Received active operations: {:?}", ops);
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn activate_operation(
        controller_id: &str,
        operation_id: &str,
        parameters: Option<&AnySerializable>,
    ) {
        let result = AUTONOMIC_CLIENT
            .activate(controller_id, operation_id, parameters)
            .await;
        match result {
            Ok(_) => {
                tracing::info!("Activation succeeded");
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn activate_operation_stream(
        controller_id: &str,
        operation_id: &str,
        parameters: Option<&AnySerializable>,
    ) {
        let result = AUTONOMIC_CLIENT
            .activate_stream(controller_id, operation_id, parameters)
            .await;
        match result {
            Ok(mut stream) => {
                while let Some(state) = stream.next().await {
                    match state {
                        OpState::Active => {
                            tracing::info!("Operation is Active");
                        }
                        OpState::Ok(val) => match val {
                            Some(msg) => {
                                tracing::info!(
                                    "Operation completed successfully with message: {}",
                                    msg
                                );
                            }
                            None => {
                                tracing::info!("Operation completed successfully");
                            }
                        },
                        OpState::Failed(val) => match val {
                            Some(msg) => {
                                tracing::info!("Operation failed with message: {}", msg);
                            }
                            None => {
                                tracing::info!("Operation failed");
                            }
                        },
                        OpState::Locked(val) => match val {
                            Some(msg) => {
                                tracing::error!("Operation locked with message: {}", msg)
                            }
                            None => {
                                tracing::error!("Operation locked");
                            }
                        },
                        OpState::Aborted => {
                            tracing::warn!("Operation aborted");
                        }
                        OpState::Panicked(message) => {
                            tracing::error!("Operation panicked with message: {}", message);
                        }
                    }
                }
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn abort_operation(controller_id: &str, operation_id: &str) {
        let result = AUTONOMIC_CLIENT.abort(controller_id, operation_id).await;
        match result {
            Ok(_) => {
                tracing::info!("Operation aborted successfully");
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn lock_operation(controller_id: &str, operation_id: &str) {
        let result = AUTONOMIC_CLIENT.lock(controller_id, operation_id).await;
        match result {
            Ok(_) => {
                tracing::info!("Operation locked successfully");
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn unlock_operation(controller_id: &str, operation_id: &str) {
        let result = AUTONOMIC_CLIENT.unlock(controller_id, operation_id).await;
        match result {
            Ok(_) => {
                tracing::info!("Operation unlocked successfully");
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn activate_sensor(controller_id: &str, operation_id: &str) {
        let result = AUTONOMIC_CLIENT
            .activate_sensor(controller_id, operation_id)
            .await;
        match result {
            Ok(_) => {
                tracing::info!("Sensor activation succeeded");
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn deactivate_sensor(controller_id: &str, operation_id: &str) {
        let result = AUTONOMIC_CLIENT
            .deactivate_sensor(controller_id, operation_id)
            .await;
        match result {
            Ok(_) => {
                tracing::info!("Sensor deactivation succeeded.");
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }
}
