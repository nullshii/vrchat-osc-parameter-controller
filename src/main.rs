use crate::osc::{OscNotification, run_osc_loop};

pub mod osc;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<OscNotification>(32);

    let _ = tokio::spawn(async move {
        run_osc_loop(tx).await;
    });

    while let Some(notification) = rx.recv().await {
        match notification {
            OscNotification::AvatarParametersUpdated {
                address,
                parameters,
            } => {
                log::info!(
                    "[Main] Caught update from {}! Params: {:?}",
                    address,
                    parameters
                );
                // >> DO YOUR MAIN THREAD LOGIC / HEART RATE ADJUSTMENTS HERE <<
            }
        }
    }

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C signal");
}
