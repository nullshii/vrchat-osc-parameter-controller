use crate::{
    app::OscApp,
    osc::{OscNotification, run_osc_loop},
};

pub mod app;
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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let app = OscApp::new();
    let shared_parameters = app.parameters.clone();

    eframe::run_native(
        "My egui App",
        options,
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();

            tokio::spawn(async move {
                while let Some(notification) = rx.recv().await {
                    match notification {
                        OscNotification::AvatarParametersUpdated {
                            address,
                            parameters,
                        } => {
                            log::info!(
                                "[Main Task] Caught update from {}! Params: {:?}",
                                address,
                                parameters
                            );

                            if let Ok(mut guard) = shared_parameters.lock() {
                                *guard = Some(parameters);
                            }

                            ctx.request_repaint();
                        }
                    }
                }
            });

            Ok(Box::new(app))
        }),
    )
    .unwrap();
}
