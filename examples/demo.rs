//! Minimal eframe window showing the crate is wired up.

use egui_pc98_revival::FKey;

const FKEY_ITEMS: [FKey<'static>; 10] = [
    FKey { key: "F1", label: "Help" },
    FKey { key: "F2", label: "Open" },
    FKey { key: "F3", label: "Save" },
    FKey { key: "F4", label: "Close" },
    FKey { key: "F5", label: "Refresh" },
    FKey { key: "F6", label: "Copy" },
    FKey { key: "F7", label: "Move" },
    FKey { key: "F8", label: "Delete" },
    FKey { key: "F9", label: "Rename" },
    FKey { key: "F10", label: "Quit" },
];

#[derive(Default)]
struct DemoApp {
    clicked_count: u32,
    checked: bool,
    status: String,
}

impl eframe::App for DemoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui_pc98_revival::ensure(ui.ctx());

        egui::Panel::top("header")
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                egui_pc98_revival::header_bar(ui, "egui PC-98 Revival", None);
            });

        egui::Panel::bottom("fkeys")
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                if let Some(index) = egui_pc98_revival::fkey_bar(ui, &FKEY_ITEMS) {
                    let item = FKEY_ITEMS[index];
                    self.status = format!("{} {} pressed", item.key, item.label);
                    if item.key == "F10" {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });

        egui::CentralPanel::default().show(ui, |ui| {
            egui_pc98_revival::panel(ui, "INFO", |ui| {
                if ui.button("Click me").clicked() {
                    self.clicked_count += 1;
                }
                ui.label(format!("Clicked {} times", self.clicked_count));
                ui.checkbox(&mut self.checked, "Toggle me");
                ui.label(&self.status);
            });
        });
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("egui PC-98 Revival")
            .with_inner_size([960.0, 600.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "egui PC-98 Revival",
        native_options,
        Box::new(|cc| {
            egui_pc98_revival::apply(&cc.egui_ctx);
            Ok(Box::new(DemoApp::default()))
        }),
    )
}
