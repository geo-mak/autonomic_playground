mod openapi;

use tracing::subscriber::set_global_default;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};

use crate::openapi::PlaygroundOpenAPIClient;
use autonomic_core::operation::OperationResult;
use autonomic_core::serde::IntoAnySerializable;
use autonomic_playground_kit::operations::{Play, PlayParameters};

pub static CONTROLLER: &str = "controller";
pub static MAIN_OPERATION: &str = "main_operation";
pub static SECONDARY_OPERATION: &str = "secondary_operation";

#[tokio::main]
async fn main() {
    // Global tracing subscriber configuration to print events to the console
    // This should be done only once per application, otherwise it will panic
    let filter = tracing_subscriber::filter::EnvFilter::new("autonomic=trace");
    let layer = tracing_subscriber::fmt::layer().with_filter(filter);
    let subscriber = Registry::default().with(layer);
    set_global_default(subscriber).expect("Failed to set global tracing subscriber");

    // Our basic Playbook
    // Before we start, the server must be up and running with required operations and sensors.

    // We start by defining basic parameters for the main operation.

    // Ok parameters will return `Ok` with a message.
    let ok_params = PlayParameters::new(
        Play::NormalResult(OperationResult::ok_str("Welcome to autonomic!")),
        None,
        0,
    )
    .into_any_serializable();

    // Err parameters will return `Err` with a message.
    let err_params = PlayParameters::new(
        Play::NormalResult(OperationResult::err_str("Expected Error")),
        None,
        0,
    )
    .into_any_serializable();

    // Panic parameters will trigger a panic.
    let panic_params = PlayParameters::new(Play::Panic, None, 0).into_any_serializable();

    // Now, we activate main operation with parameters that will trigger a panic.
    // Panics are considered unexpected errors, and they will lock the operation.
    // Operation will be locked after panic, and any further activation will be rejected.
    // If the operation has an active sensor, it will be deactivated also when the operation is locked.
    // Locking the operation is a safety measure to prevent further panics, and eventually any unwanted behavior.
    PlaygroundOpenAPIClient::activate_operation_stream(
        CONTROLLER,
        MAIN_OPERATION,
        Some(&panic_params),
    )
    .await;

    // By now, the main operation should have been locked, so we can unlock it.
    // Operations can be unlocked manually later, presumably after fixing the issue that caused the panic.
    PlaygroundOpenAPIClient::unlock_operation(CONTROLLER, MAIN_OPERATION).await;

    // Since the operation is unlocked, we can activate it again.
    // If the operation has been assigned a sensor, it needs to be reactivated as well.
    // This time, we will activate it with parameters that will return `Ok` with a message.
    PlaygroundOpenAPIClient::activate_operation_stream(
        CONTROLLER,
        MAIN_OPERATION,
        Some(&ok_params),
    )
    .await;

    // Now, we will activate the main operation with parameters that will return `Err` with a message.
    // Expected errors that are returned by the operation are considered normal behavior.
    // The operation will not be locked, and it can be activated again, even with the same parameters.
    // If the operation has an active sensor, it will remain active.
    PlaygroundOpenAPIClient::activate_operation_stream(
        CONTROLLER,
        MAIN_OPERATION,
        Some(&err_params),
    )
    .await;

    // We will activate the sensor associated with the main operation.
    // Each operation can have only one sensor, and it can be deactivated at any time.
    // Sensor encapsulates an activation condition with optional parameters for the operation.
    // Activation conditions are trait-based, and they can be implemented by the user.
    // Activation condition can be a simple interval, or a complex condition that requires multiple events.
    // When the operation is locked, the sensor will be deactivated as well.
    // When the operation is unlocked, the sensor will remain deactivated until it is activated again.
    PlaygroundOpenAPIClient::activate_sensor(CONTROLLER, MAIN_OPERATION).await;

    // We wait for 2 seconds to see the sensor in action.
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Finally, we deactivate the sensor associated with the main operation.
    PlaygroundOpenAPIClient::deactivate_sensor(CONTROLLER, MAIN_OPERATION).await;
}
