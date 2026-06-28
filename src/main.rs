use crate::{
    app::OscApp,
    osc::{OscNotification, run_osc_loop},
    osc_node_flatten::flatten_osc_nodes,
};

pub mod app;
pub mod osc;
pub mod osc_node_flatten;

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
                            address: _,
                            parameters,
                        } => {
                            let mut flat_list = Vec::new();
                            flatten_osc_nodes(&parameters, &mut flat_list);

                            // Sort alphabetically by name for a cleaner UI experience
                            flat_list.sort_by(|a, b| a.address.cmp(&b.address));

                            // Update your App state (ensure app.parameters is Arc<Mutex<Vec<FlatParameter>>>)
                            if let Ok(mut guard) = shared_parameters.lock() {
                                *guard = flat_list;
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
