use std::sync::{Arc, Mutex};

use vrchat_osc::models::AccessMode;

use crate::osc_node_flatten::{ElementValue, OscElement};

pub struct OscApp {
    pub parameters: Arc<Mutex<Vec<OscElement>>>,
}

impl eframe::App for OscApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("VRChat OSC Controller");
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                if let Ok(mut parameters) = self.parameters.try_lock() {
                    if parameters.is_empty() {
                        ui.weak("Waiting for OSC data tree from VRChat...");
                        return;
                    }
                    egui::Grid::new("interactive_osc_grid")
                        .num_columns(3)
                        .spacing([20.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            for element in parameters.iter_mut() {
                                // Column 1: The OSC Address Path
                                ui.label(&element.address);

                                // Column 2: Access mode Badge
                                match element.access {
                                    AccessMode::ReadOnly => ui.weak("Read"),
                                    AccessMode::WriteOnly => ui.weak("Write"),
                                    AccessMode::ReadWrite => ui.weak("Read/Write"),
                                    _ => ui.weak("-"),
                                };

                                // Column 3: Control Widget (Disabled if ReadOnly)
                                let is_editable = matches!(
                                    element.access,
                                    AccessMode::ReadWrite | AccessMode::WriteOnly
                                );

                                ui.add_enabled_ui(is_editable, |ui| {
                                    let mut changed = false;

                                    match &mut element.value {
                                        ElementValue::Bool(b) => {
                                            if ui.checkbox(b, "").changed() {
                                                changed = true;
                                            }
                                        }
                                        ElementValue::Float(f) => {
                                            // Sliders work beautifully for avatar parameters (0.0 to 1.0 range usually)
                                            if ui
                                                .add(
                                                    egui::Slider::new(f, 0.0..=1.0)
                                                        .drag_value_speed(0.01),
                                                )
                                                .changed()
                                            {
                                                changed = true;
                                            }
                                        }
                                        ElementValue::Int(i) => {
                                            if ui.add(egui::DragValue::new(i)).changed() {
                                                changed = true;
                                            }
                                        }
                                        ElementValue::Unsupported(s) => {
                                            ui.weak(s);
                                        }
                                    }

                                    // Send back to VRChat if modified!
                                    if changed {
                                        log::info!(
                                            "Value changed for {}! New state: {:?}",
                                            element.address,
                                            element.value
                                        );
                                        // TODO: Pass a sender channel to your OscApp state
                                        // so you can forward this change to `vrchat_osc.send_back(...)`
                                    }
                                });

                                ui.end_row();
                            }
                        });
                } else {
                    ui.label("Accessing elements lock...");
                }
            });
    }
}

impl OscApp {
    pub fn new() -> Self {
        Self {
            parameters: Arc::new(Mutex::new(vec![])),
        }
    }
}
