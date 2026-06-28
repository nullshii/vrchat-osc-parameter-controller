use crate::osc::run_osc_loop;

pub mod osc;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let _ = tokio::spawn(async move {
        run_osc_loop().await;
    });

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C signal");
}
