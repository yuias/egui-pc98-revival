//! Full demo: menu bar, header, function-key bar, two side-by-side panels,
//! a hue swatch row and a floating window with stock egui widgets, all styled
//! through [`egui_pc98_revival`].
//!
//! Set `PC98_DEMO_ZOOM` (e.g. `1.5`) to preview other display scale factors.

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

const FILES: [&str; 25] = [
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
    "FORMAT.COM",
    "SYS.COM",
    "EDLIN.EXE",
    "DISKCOPY.EXE",
    "CHKDSK.EXE",
    "SWITCH.EXE",
    "SPEED.EXE",
    "TOUHOU.EXE",
    "MUSIC.M",
    "SOUND.BAS",
    "FM.DRV",
    "PMD.COM",
    "MOUSE.SYS",
    "EMM386.EXE",
    "HIMEM.SYS",
];

const MODES: [&str; 3] = ["640x400 16 colors", "640x400 8 colors", "640x200 8 colors"];

struct DemoApp {
    started: std::time::Instant,
    selected_file: usize,
    checked: bool,
    slider: f32,
    text: String,
    status: String,
    show_window: bool,
    mode: usize,
    drive: u8,
    count: i32,
    focused_panel: usize,
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
            show_window: true,
            mode: 0,
            drive: 0,
            count: 3,
            focused_panel: 0,
        }
    }
}

impl eframe::App for DemoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui_pc98_revival::ensure(ui.ctx());

        let clock = format!("{:.1}s", self.started.elapsed().as_secs_f32());
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(100));

        egui::Panel::top("menu").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        self.status = String::from("File > Open");
                    }
                    if ui.button("Save").clicked() {
                        self.status = String::from("File > Save");
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_window, "Settings window");
                    ui.menu_button("Mode", |ui| {
                        for (i, mode) in MODES.iter().enumerate() {
                            ui.radio_value(&mut self.mode, i, *mode);
                        }
                    });
                });
            });
        });

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
            let swatch_h = egui_pc98_revival::dots(ui.ctx(), 64.0);
            let panels_h = (ui.available_height() - swatch_h).max(0.0);

            ui.allocate_ui(egui::vec2(ui.available_width(), panels_h), |ui| {
                ui.columns(2, |columns| {
                    let files_response = egui_pc98_revival::TitledPanel::new("FILES")
                        .focused(self.focused_panel == 0)
                        .show_with_title(
                            &mut columns[0],
                            |ui| {
                                ui.label(format!("{} FILES", FILES.len()));
                            },
                            |ui| {
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
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
                            },
                        );
                    if files_response.response.clicked() {
                        self.focused_panel = 0;
                    }

                    let info_response = egui_pc98_revival::TitledPanel::new("INFO")
                        .focused(self.focused_panel == 1)
                        .show(&mut columns[1], |ui| {
                            // The font maps the JIS full-width minus to U+2212, not U+FF0D.
                            ui.label("「ＰＣ−９８０１ シリーズ」");
                            ui.label("日本電気株式会社製 パーソナルコンピュータ");
                            ui.label(egui_pc98_revival::text::truncate_tail(
                                "日本電気株式会社製 パーソナルコンピュータ",
                                20,
                            ));
                            if ui
                                .button("Run")
                                .on_hover_text("Run the selected file")
                                .clicked()
                            {
                                self.status = format!("Running {}", FILES[self.selected_file]);
                            }
                            ui.checkbox(&mut self.checked, "Enable turbo mode");
                            ui.add(egui::Slider::new(&mut self.slider, 0.0..=100.0).text("volume"));
                            ui.text_edit_singleline(&mut self.text);
                            ui.label(&self.status);
                        });
                    if info_response.response.clicked() {
                        self.focused_panel = 1;
                    }
                });
            });

            ui.add_space(egui_pc98_revival::dots(ui.ctx(), 8.0));

            ui.horizontal(|ui| {
                let swatch = egui_pc98_revival::dots(ui.ctx(), 24.0);
                for hue in egui_pc98_revival::palette::HUES {
                    let (solid_rect, _) =
                        ui.allocate_exact_size(egui::vec2(swatch, swatch), egui::Sense::hover());
                    ui.painter().rect_filled(solid_rect, 0, hue);

                    let (dither_rect, _) =
                        ui.allocate_exact_size(egui::vec2(swatch, swatch), egui::Sense::hover());
                    egui_pc98_revival::paint_dither(
                        ui.painter(),
                        dither_rect,
                        hue,
                        Some(egui_pc98_revival::palette::GROUND),
                    );
                }
            });
        });

        let progress = (self.started.elapsed().as_secs_f32() / 10.0).fract();
        egui::Window::new("SETTINGS")
            .open(&mut self.show_window)
            .default_pos([520.0, 300.0])
            .show(ui.ctx(), |ui| {
                egui::ComboBox::from_label("Display")
                    .selected_text(MODES[self.mode])
                    .show_ui(ui, |ui| {
                        for (i, mode) in MODES.iter().enumerate() {
                            ui.selectable_value(&mut self.mode, i, *mode);
                        }
                    });
                ui.horizontal(|ui| {
                    for (i, drive) in ["A:", "B:", "C:"].iter().enumerate() {
                        ui.radio_value(&mut self.drive, i as u8, *drive);
                    }
                });
                ui.add(
                    egui::DragValue::new(&mut self.count)
                        .range(1..=9)
                        .prefix("Count: "),
                );
                // ProgressBar rounds its ends unless told otherwise.
                ui.add(
                    egui::ProgressBar::new(progress)
                        .show_percentage()
                        .corner_radius(0),
                );
                ui.separator();
                ui.collapsing("Details", |ui| {
                    ui.label("Memory: 640KB");
                    ui.label("FPU: none");
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
            if let Some(zoom) = std::env::var("PC98_DEMO_ZOOM")
                .ok()
                .and_then(|z| z.parse::<f32>().ok())
            {
                cc.egui_ctx.set_zoom_factor(zoom);
            }
            egui_pc98_revival::apply(&cc.egui_ctx);
            Ok(Box::new(DemoApp::default()))
        }),
    )
}
