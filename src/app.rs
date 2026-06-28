use std::sync::{Arc, Mutex};

use vrchat_osc::models::OscNode;

pub struct OscApp {
    pub parameters: Arc<Mutex<Option<OscNode>>>,
}

impl eframe::App for OscApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("VRChat OSC Controller");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Ok(guard) = self.parameters.try_lock() {
                // Iterate over references inside the vector
                for node in guard.iter() {
                    ui.horizontal(|ui| {
                        ui.label(&node.full_path);
                    });
                }
            } else {
                ui.label("Loading parameters...");
            }
        });
    }
}

impl OscApp {
    pub fn new() -> Self {
        Self {
            parameters: Arc::new(Mutex::new(None::<OscNode>)),
        }
    }
}
