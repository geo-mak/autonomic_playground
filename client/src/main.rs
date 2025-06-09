mod openapi;

use std::time::Duration;

use tracing::subscriber::set_global_default;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};

use crate::openapi::controller::{CTRL_1, PGControllerClient, STATE_STORE_1};

#[tokio::main]
async fn main() {
    // Global tracing subscriber configuration to print events to the console
    // This should be done only once per process, otherwise it will panic
    let filter = tracing_subscriber::filter::EnvFilter::new("autonomic=trace,client=trace");
    let layer = tracing_subscriber::fmt::layer().with_filter(filter);
    let subscriber = Registry::default().with(layer);
    set_global_default(subscriber).expect("Failed to set global tracing subscriber");

    // Discovery.
    PGControllerClient::list().await;

    // This will trigger the sensors and start the control operation of the observing controller.
    PGControllerClient::change_observed_state(STATE_STORE_1, "play").await;

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Manual triggering of the control operation.
    PGControllerClient::perform(CTRL_1).await;
}
