//! Full demo: menu bar, header, function-key bar, two side-by-side panels,
//! a hue swatch row and a floating window with stock egui widgets, all styled
//! through [`egui_pc98_revival`].
//!
//! Set `PC98_DEMO_ZOOM` (e.g. `1.5`) to preview other display scale factors.

use egui_pc98_revival::{Column, ColumnWidth, Dialog, FKey, ListState, ListView, TabStyle};

const HELP_KEYS: [(&str, &str); 5] = [
    ("F1", "Show this help"),
    ("Up/Down", "Move the cursor"),
    ("PgUp/PgDn", "Move by a page"),
    ("Enter", "Open the selected file"),
    ("Esc", "Close a dialog"),
];

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
    drive_tab: usize,
    info_tab: usize,
    volume: f32,
    files_list: ListState,
    show_help: bool,
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
            drive_tab: 0,
            info_tab: 0,
            volume: 0.6,
            files_list: ListState::default(),
            show_help: false,
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
                ui.menu_button("Help", |ui| {
                    if ui.button("Keys...").clicked() {
                        self.show_help = true;
                        ui.close();
                    }
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
                    if item.key == "F1" {
                        self.show_help = true;
                    }
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
                                egui_pc98_revival::tab_strip(
                                    ui,
                                    &mut self.drive_tab,
                                    &["A:", "B:", "C:"],
                                    TabStyle::TitleStrip,
                                );
                            },
                            |ui| {
                                // Row 0 is a directory; the rest are files indented under it.
                                let columns = [
                                    Column::new("NAME", ColumnWidth::Fill { min_cells: 12 }),
                                    Column::new("SIZE", ColumnWidth::Cells(7))
                                        .align(egui::Align::Max)
                                        .drop_priority(1),
                                    Column::new("DATE", ColumnWidth::Cells(8)).drop_priority(2),
                                ];
                                let list_response = ListView::new("files_list", FILES.len() + 1)
                                    .columns(&columns)
                                    .show(ui, &mut self.files_list, |row, i| {
                                        if i == 0 {
                                            row.cell(0, "DOS\\", None);
                                        } else {
                                            let index = i - 1;
                                            row.indent(1);
                                            row.cell(0, FILES[index], None);
                                            // Fake size/date derived from the row index.
                                            let size = 512 + (index * 1237) % 90_000;
                                            row.cell(1, &size.to_string(), None);
                                            let month = index % 12 + 1;
                                            let day = index % 28 + 1;
                                            row.cell(2, &format!("{month:02}-{day:02}"), None);
                                        }
                                    });
                                if list_response.cursor_changed && self.files_list.cursor > 0 {
                                    self.selected_file = self.files_list.cursor - 1;
                                }
                                if let Some(row) = list_response.activated
                                    && row > 0
                                {
                                    self.status = format!("Opened {}", FILES[row - 1]);
                                }
                            },
                        );
                    if files_response.response.clicked() {
                        self.focused_panel = 0;
                    }

                    let info_response = egui_pc98_revival::TitledPanel::new("INFO")
                        .focused(self.focused_panel == 1)
                        .show(&mut columns[1], |ui| {
                            egui_pc98_revival::tab_strip(
                                ui,
                                &mut self.info_tab,
                                &["GENERAL", "SOUND"],
                                TabStyle::Bar,
                            );
                            if self.info_tab == 0 {
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
                                ui.add(
                                    egui::Slider::new(&mut self.slider, 0.0..=100.0).text("volume"),
                                );
                                ui.text_edit_singleline(&mut self.text);
                                ui.label(&self.status);
                            } else {
                                let palette = egui_pc98_revival::palette(ui.ctx());
                                // Triangle wave, 2-second period.
                                let phase = (self.started.elapsed().as_secs_f32() / 2.0).fract();
                                let level = if phase < 0.5 {
                                    phase * 2.0
                                } else {
                                    2.0 - phase * 2.0
                                };

                                ui.horizontal(|ui| {
                                    egui_pc98_revival::SegmentBar::new(12, level)
                                        .warn(10, palette.warn)
                                        .show(ui);
                                    egui_pc98_revival::SegmentBar::new(12, level)
                                        .warn(10, palette.warn)
                                        .vertical(true)
                                        .show(ui);
                                });

                                egui_pc98_revival::SegmentBar::new(10, 0.0)
                                    .fill(palette.frame)
                                    .warn(8, palette.accent)
                                    .show_interactive(ui, &mut self.volume);
                                ui.label(format!("Volume: {:.0}%", self.volume * 100.0));
                            }
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

        if self.show_help {
            let palette = egui_pc98_revival::palette(ui.ctx());
            let size = egui::vec2(
                egui_pc98_revival::dots(ui.ctx(), 280.0),
                egui_pc98_revival::dots(ui.ctx(), 200.0),
            );
            let result =
                Dialog::new(egui::Id::new("help"), "HELP")
                    .size(size)
                    .show(ui.ctx(), |ui| {
                        for (key, desc) in HELP_KEYS {
                            ui.horizontal(|ui| {
                                ui.colored_label(palette.accent, key);
                                ui.label(desc);
                            });
                        }
                        ui.add_space(egui_pc98_revival::dots(ui.ctx(), 8.0));
                        egui_pc98_revival::fkey_bar(
                            ui,
                            &[FKey {
                                key: "ESC",
                                label: "Close",
                            }],
                        )
                        .is_some()
                    });
            if result.close_requested || result.inner {
                self.show_help = false;
            }
        }
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
