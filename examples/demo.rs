//! Full demo: header, function-key bar, two side-by-side panels and a hue
//! swatch row, all styled through [`egui_pc98_revival`].

use egui_pc98_revival::FKey;

const FKEY_ITEMS: [FKey<'static>; 10] = [
    FKey {
        key: "F1",
        label: "Help",
    },
    FKey {
        key: "F2",
        label: "Open",
    },
    FKey {
        key: "F3",
        label: "Save",
    },
    FKey {
        key: "F4",
        label: "Close",
    },
    FKey {
        key: "F5",
        label: "Refresh",
    },
    FKey {
        key: "F6",
        label: "Copy",
    },
    FKey {
        key: "F7",
        label: "Move",
    },
    FKey {
        key: "F8",
        label: "Delete",
    },
    FKey {
        key: "F9",
        label: "Rename",
    },
    FKey {
        key: "F10",
        label: "Quit",
    },
];

const FILES: [&str; 10] = [
    "AUTOEXEC.BAT",
    "CONFIG.SYS",
    "COMMAND.COM",
    "MSDOS.SYS",
    "IO.SYS",
    "GAME.EXE",
    "DATA.DAT",
    "README.TXT",
    "BASIC.EXE",
    "N88BASIC.EXE",
];

struct DemoApp {
    started: std::time::Instant,
    selected_file: usize,
    checked: bool,
    slider: f32,
    text: String,
    status: String,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self {
            started: std::time::Instant::now(),
            selected_file: 0,
            checked: false,
            slider: 50.0,
            text: String::from("N88-BASIC(86)"),
            status: String::from("Ready."),
        }
    }
}

impl eframe::App for DemoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui_pc98_revival::ensure(ui.ctx());

        let clock = format!("{:.1}s", self.started.elapsed().as_secs_f32());
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(100));

        egui::Panel::top("header")
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                egui_pc98_revival::header_bar(ui, "egui PC-98 Revival", Some(&clock));
            });

        egui::Panel::bottom("fkeys")
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                if let Some(index) = egui_pc98_revival::fkey_bar(ui, &FKEY_ITEMS) {
                    let item = FKEY_ITEMS[index];
                    self.status = format!("{} {} pressed", item.key, item.label);
                    if item.key == "F10" {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let swatch_h = 64.0;
            let panels_h = (ui.available_height() - swatch_h).max(0.0);

            ui.allocate_ui(egui::vec2(ui.available_width(), panels_h), |ui| {
                ui.columns(2, |columns| {
                    egui_pc98_revival::panel(&mut columns[0], "FILES", |ui| {
                        for (i, name) in FILES.iter().enumerate() {
                            if ui
                                .selectable_label(self.selected_file == i, *name)
                                .clicked()
                            {
                                self.selected_file = i;
                                self.status = format!("Selected {name}");
                            }
                        }
                    });

                    egui_pc98_revival::panel(&mut columns[1], "INFO", |ui| {
                        // The font maps the JIS full-width minus to U+2212, not U+FF0D.
                        ui.label("「ＰＣ−９８０１ シリーズ」");
                        ui.label("日本電気株式会社製 パーソナルコンピュータ");
                        if ui.button("Run").clicked() {
                            self.status = format!("Running {}", FILES[self.selected_file]);
                        }
                        ui.checkbox(&mut self.checked, "Enable turbo mode");
                        ui.add(egui::Slider::new(&mut self.slider, 0.0..=100.0).text("volume"));
                        ui.text_edit_singleline(&mut self.text);
                        ui.label(&self.status);
                    });
                });
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                for hue in egui_pc98_revival::palette::HUES {
                    let (solid_rect, _) =
                        ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                    ui.painter().rect_filled(solid_rect, 0, hue);

                    let (dither_rect, _) =
                        ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                    egui_pc98_revival::paint_dither(
                        ui.painter(),
                        dither_rect,
                        hue,
                        Some(egui_pc98_revival::palette::GROUND),
                    );
                }
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
