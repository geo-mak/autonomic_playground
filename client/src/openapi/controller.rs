use lazy_static::lazy_static;
use reqwest::Client;
use tokio_stream::StreamExt;

use autonomic_api::controller::ControllerClient;
use autonomic_controllers::controller::OpState;
use autonomic_service::openapi::client::OpenAPIClient;

lazy_static! {
    pub static ref AUTONOMIC_CLIENT: OpenAPIClient<'static> =
        OpenAPIClient::new(Client::new(), "http://127.0.0.1:8000");
}

#[allow(dead_code)]
pub static STATE_STORE_1: &str = "store_1";

#[allow(dead_code)]
pub static STATE_STORE_2: &str = "store_2";

#[allow(dead_code)]
pub static STATE_STORE_3: &str = "store_3";

#[allow(dead_code)]
pub static CTRL_1: &str = "controller_1";

#[allow(dead_code)]
pub static CTRL_2: &str = "controller_2";

#[allow(dead_code)]
pub static CTRL_3: &str = "controller_3";

pub struct PGControllerClient;

#[allow(dead_code)]
impl PGControllerClient {
    pub async fn ctrl(controller_id: &str) {
        let result = AUTONOMIC_CLIENT.ctrl(controller_id).await;
        match result {
            Ok(ctrl_info) => {
                tracing::info!(
                    "Received info about controller {}: {:?}",
                    controller_id,
                    ctrl_info
                );
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn list() {
        match AUTONOMIC_CLIENT.list().await {
            Ok(ctrl_infos) => {
                tracing::info!("Controllers:");
                tracing::info!(
                    "{:<15} | {:<30} | {:<10} | {:<10} | {:<10}",
                    "ID",
                    "Description",
                    "Performing",
                    "Locked",
                    "Sensing"
                );
                tracing::info!("{}", "-".repeat(85));
                for ctrl in ctrl_infos {
                    tracing::info!(
                        "{:<15} | {:<30} | {:<10} | {:<10} | {:<10}",
                        ctrl.id(),
                        ctrl.description(),
                        ctrl.performing(),
                        ctrl.locked(),
                        ctrl.sensing(),
                    );
                }
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn list_performing() {
        match AUTONOMIC_CLIENT.list_performing().await {
            Ok(ctrls) => {
                tracing::info!("Received active operations: {:?}", ctrls);
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn perform(controller_id: &str) {
        match AUTONOMIC_CLIENT.perform(controller_id).await {
            Ok(mapper) => {
                let mut stream = mapper.map();
                while let Some(state) = stream.next().await {
                    match state {
                        OpState::Started => {
                            tracing::info!("Operation started");
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

    pub async fn abort(controller_id: &str) {
        let result = AUTONOMIC_CLIENT.abort(controller_id).await;
        match result {
            Ok(_) => {
                tracing::info!("Operation aborted successfully");
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn lock(controller_id: &str) {
        let result = AUTONOMIC_CLIENT.lock(controller_id).await;
        match result {
            Ok(_) => {
                tracing::info!("Controller {} locked successfully", controller_id);
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn unlock(controller_id: &str) {
        let result = AUTONOMIC_CLIENT.unlock(controller_id).await;
        match result {
            Ok(_) => {
                tracing::info!("Controller {} unlocked successfully", controller_id);
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn start_sensor(controller_id: &str) {
        let result = AUTONOMIC_CLIENT.start_sensor(controller_id).await;
        match result {
            Ok(_) => {
                tracing::info!(
                    "Sensor for controller {} started successfully",
                    controller_id
                );
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn stop_sensor(controller_id: &str) {
        let result = AUTONOMIC_CLIENT.stop_sensor(controller_id).await;
        match result {
            Ok(_) => {
                tracing::info!(
                    "Sensor for controller {} stopped successfully",
                    controller_id
                );
            }
            Err(err) => {
                tracing::error!("Client Error: {:?}", err);
            }
        }
    }

    pub async fn change_observed_state(store: &str, value: &str) {
        let url = format!("http://127.0.0.1:8000/change_state/{}", store);
        let client = reqwest::Client::new();
        let result = client.post(&url).json(&value).send().await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("Observed state for '{}' changed successfully", store);
            }
            Ok(resp) => {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                tracing::error!(
                    "Failed to change observed state for '{}': {} - {}",
                    store,
                    status,
                    text
                );
            }
            Err(err) => {
                tracing::error!(
                    "HTTP error while changing observed state for '{}': {:?}",
                    store,
                    err
                );
            }
        }
    }
}
