use crate::{
    app::OscApp,
    osc::{OscNotification, run_osc_loop},
    osc_node_flatten::{ElementValue, flatten_osc_nodes},
};

pub mod app;
pub mod osc;
pub mod osc_node_flatten;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
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
        "VRChat OSC Controller",
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
                            flat_list.sort_by(|a, b| {
                                let access_priority =
                                    |mode: &vrchat_osc::models::AccessMode| match mode {
                                        vrchat_osc::models::AccessMode::ReadWrite => 0,
                                        vrchat_osc::models::AccessMode::ReadOnly => 1,
                                        vrchat_osc::models::AccessMode::WriteOnly => 2,
                                        _ => 3, // AccessMode::None or others
                                    };

                                let weight_a = access_priority(&a.access);
                                let weight_b = access_priority(&b.access);

                                match weight_a.cmp(&weight_b) {
                                    std::cmp::Ordering::Equal => {
                                        // 3. Secondary Sort: If access modes are identical,
                                        // sort alphabetically by address so it stays clean!
                                        a.address.cmp(&b.address)
                                    }
                                    other => other,
                                }
                            });

                            // Update your App state (ensure app.parameters is Arc<Mutex<Vec<FlatParameter>>>)
                            if let Ok(mut guard) = shared_parameters.lock() {
                                *guard = flat_list;
                            }

                            ctx.request_repaint();
                        }
                        OscNotification::PacketReceived { address: _, packet } => {
                            if let Ok(mut elements) = shared_parameters.lock() {
                                if let rosc::OscPacket::Message(msg) = packet {
                                    if let Some(first_arg) = msg.args.first() {
                                        let incoming_value = match first_arg {
                                            rosc::OscType::Bool(b) => Some(ElementValue::Bool(*b)),
                                            rosc::OscType::Float(f) => {
                                                Some(ElementValue::Float(*f as f64))
                                            }
                                            rosc::OscType::Int(i) => Some(ElementValue::Int(*i)),
                                            _ => None,
                                        };

                                        if let Some(new_val) = incoming_value {
                                            if let Some(element) =
                                                elements.iter_mut().find(|e| e.address == msg.addr)
                                            {
                                                element.value = new_val;
                                            }
                                        }
                                    }
                                }
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
