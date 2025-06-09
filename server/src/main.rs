use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use tracing::subscriber::set_global_default;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};

use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};

use pg_kit::controller::PGController;

use autonomic_controllers::provider::ControllerManager;
use autonomic_service::openapi::router::controller_router;
use autonomic_service::openapi::server::OpenAPIServer;

static DEFAULT_STATE: &str = "default state";
static CTRL_DESC: &str = "Play ground test controller";

pub async fn change_state(
    Path(store): Path<String>,
    Json(content): Json<String>,
) -> impl IntoResponse {
    match tokio::fs::write(&store, content).await {
        Ok(_) => (axum::http::StatusCode::OK, "State updated").into_response(),
        Err(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to write file",
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() {
    // One time setup, or it will panic.
    let filter =
        tracing_subscriber::filter::EnvFilter::new("autonomic=trace,server=trace,pg_kit=trace");
    let layer = tracing_subscriber::fmt::layer().with_filter(filter);
    let subscriber = Registry::default().with(layer);
    set_global_default(subscriber).expect("Failed to set global tracing subscriber");

    // The provider component.
    let mut provider = ControllerManager::new();

    // How fast the checking should be in this setup.
    let poll_speed = Duration::from_secs(1);

    let controller_1 = PGController::new(
        "controller_1",
        CTRL_DESC,
        "store_1",
        DEFAULT_STATE,
        poll_speed,
    );
    let controller_2 = PGController::new(
        "controller_2",
        CTRL_DESC,
        "store_2",
        DEFAULT_STATE,
        poll_speed,
    );
    let controller_3 = PGController::new(
        "controller_3",
        CTRL_DESC,
        "store_3",
        DEFAULT_STATE,
        poll_speed,
    );

    provider.submit(controller_1);
    provider.submit(controller_2);
    provider.submit(controller_3);

    let static_provider = provider.into_static();

    // Fires the sensors of all controllers.
    static_provider.start();

    let ctrl_rt = controller_router(static_provider);
    let update_rt = Router::new().route("/change_state/:store", post(change_state));
    let merged_rt = ctrl_rt.merge(update_rt);

    let api_server = OpenAPIServer::new();

    api_server
        .serve(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000),
            merged_rt,
        )
        .await;
}
