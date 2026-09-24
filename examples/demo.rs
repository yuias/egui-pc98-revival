//! Minimal eframe window showing the crate is wired up.

struct DemoApp;

impl eframe::App for DemoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("egui PC-98 Revival");
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
        Box::new(|_cc| Ok(Box::new(DemoApp))),
    )
}
