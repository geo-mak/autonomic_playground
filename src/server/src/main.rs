use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tracing::subscriber::set_global_default;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};

use autonomic_core::controller::OperationController;
use autonomic_core::operation::OperationResult;
use autonomic_core::traits::{IntoArc, IntoSensor};
use autonomic_service::openapi::router::controller_router;
use autonomic_service::openapi::server::OpenAPIServer;

use autonomic_playground_kit::conditions::Interval;
use autonomic_playground_kit::operations::{Play, PlayParameters, PlaygroundOperation};

pub static MAIN_CONTROLLER: &str = "controller";
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

    // New operation we consider to be the main operation
    let main_operation = PlaygroundOperation::new(MAIN_OPERATION, "Main playground operation");

    // Parameters of the activation condition
    let condition_params = PlayParameters::new(Play::NormalResult(OperationResult::Ok), None, 0);

    // A new sensor that will trigger the main operation every second.
    let sensor = Interval::new(2, Some(condition_params.into_arc())).into_sensor();

    // New operation we consider to be the secondary operation
    let secondary_operation =
        PlaygroundOperation::new(SECONDARY_OPERATION, "Secondary playground operation");

    // New controller instance
    let mut controller = OperationController::new(MAIN_CONTROLLER);

    // Submitting main operation with sensor with parameters
    controller.submit_parameters::<PlayParameters>(main_operation, Some(sensor));

    // Submitting secondary operation without sensor
    // Since parameters are of the same type as the main operation, we don't need to specify them
    controller.submit(secondary_operation, None);

    // New router instance
    let router = controller_router(controller.into_arc());

    // New server instance
    let server = OpenAPIServer::new();

    // Serving
    server
        .serve(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000),
            router,
        )
        .await;
}
