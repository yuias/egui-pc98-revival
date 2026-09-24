//! Minimal eframe window showing the crate is wired up.

#[derive(Default)]
struct DemoApp {
    clicked_count: u32,
    checked: bool,
}

impl eframe::App for DemoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui_pc98_revival::ensure(ui.ctx());

        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("egui PC-98 Revival");
            if ui.button("Click me").clicked() {
                self.clicked_count += 1;
            }
            ui.label(format!("Clicked {} times", self.clicked_count));
            ui.checkbox(&mut self.checked, "Toggle me");
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
