//! Function-key bar: a row of key/label slots, PC-98 style.

use egui::{Rect, Sense, Ui, pos2, vec2};

use crate::style::{dot, dots, palette};

/// One function-key slot: key caption, action label, optional keyboard binding.
#[derive(Clone, Copy, Debug)]
pub struct FKey<'a> {
    pub key: &'a str,
    pub label: &'a str,
    /// Pressing this returns the slot index from `fkey_bar`, like a click.
    pub shortcut: Option<egui::KeyboardShortcut>,
    /// Disabled slots draw dimmed and ignore clicks and the shortcut.
    pub enabled: bool,
}

impl<'a> FKey<'a> {
    /// Enabled slot without a shortcut.
    pub const fn new(key: &'a str, label: &'a str) -> Self {
        Self {
            key,
            label,
            shortcut: None,
            enabled: true,
        }
    }

    /// Binds a shortcut (with modifiers).
    pub const fn shortcut(mut self, shortcut: egui::KeyboardShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Binds `key` without modifiers.
    pub const fn key_shortcut(self, key: egui::Key) -> Self {
        self.shortcut(egui::KeyboardShortcut::new(egui::Modifiers::NONE, key))
    }

    /// Sets whether the slot accepts clicks and its shortcut.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Function-key bar. Returns the index of the slot clicked, or whose
/// shortcut was pressed, this frame.
pub fn fkey_bar(ui: &mut Ui, items: &[FKey<'_>]) -> Option<usize> {
    let palette = palette(ui.ctx());
    let dot = dot(ui.ctx());
    let font_id = egui::TextStyle::Body.resolve(ui.style());
    let inset = dots(ui.ctx(), 4.0);

    let outer = ui.available_rect_before_wrap();
    let painter = ui.painter();

    let key_galleys: Vec<_> = items
        .iter()
        .map(|item| painter.layout_no_wrap(item.key.to_owned(), font_id.clone(), palette.dim))
        .collect();
    let label_galleys: Vec<_> = items
        .iter()
        .map(|item| {
            let color = if item.enabled {
                palette.fkey_fg
            } else {
                palette.dim
            };
            painter.layout_no_wrap(item.label.to_owned(), font_id.clone(), color)
        })
        .collect();

    let key_widths: Vec<f32> = key_galleys.iter().map(|g| g.size().x).collect();
    let longest_label_w = label_galleys
        .iter()
        .map(|g| g.size().x)
        .fold(0.0f32, f32::max);
    let text_height = key_galleys
        .iter()
        .chain(label_galleys.iter())
        .map(|g| g.size().y)
        .fold(0.0f32, f32::max);

    let height = text_height + 2.0 * (dots(ui.ctx(), 2.0) + 2.0 * dot);
    let bar_rect = Rect::from_min_size(outer.min, vec2(ui.available_width(), height));
    painter.rect_filled(bar_rect, 0, palette.ground);

    let n = fit_fkey_items(bar_rect.width(), &key_widths, longest_label_w, inset);

    let mut clicked = None;
    if n > 0 {
        let slot_w = bar_rect.width() / n as f32;
        for i in 0..n {
            let slot = Rect::from_min_size(
                pos2(bar_rect.min.x + i as f32 * slot_w, bar_rect.min.y),
                vec2(slot_w, height),
            );
            let enabled = items[i].enabled;
            let sense = if enabled {
                Sense::click()
            } else {
                Sense::hover()
            };
            let response = ui.interact(slot, ui.id().with(("pc98_fkey", i)), sense);
            if response.clicked() && clicked.is_none() {
                clicked = Some(i);
            }

            let key_galley = &key_galleys[i];
            let key_pos = pos2(
                slot.left() + inset,
                slot.center().y - key_galley.size().y / 2.0,
            );
            painter.galley(key_pos, key_galley.clone(), palette.dim);
            let key_right = key_pos.x + key_galley.size().x;

            let label_rect = Rect::from_min_max(
                pos2(key_right + inset, slot.top() + dots(ui.ctx(), 2.0)),
                pos2(slot.right() - inset, slot.bottom() - dots(ui.ctx(), 2.0)),
            );
            let (label_bg, label_fg) = if !enabled {
                (palette.well, palette.dim)
            } else if response.hovered() {
                (palette.accent, palette.fkey_fg)
            } else {
                (palette.fkey_bg, palette.fkey_fg)
            };
            painter.rect_filled(label_rect, 0, label_bg);

            let label_galley = &label_galleys[i];
            let label_pos = label_rect.center() - label_galley.size() / 2.0;
            painter.galley(label_pos, label_galley.clone(), label_fg);
        }
    }

    // Shortcuts are checked over *all* items, including slots dropped for
    // width, so a keyboard binding never depends on window width. Only
    // checked when this layer can take input, so a dialog's own fkey bar
    // gets F-keys instead of the screen behind it (the modal-layer check
    // reads the previous frame's bookkeeping, so it lags one frame).
    let shortcuts_allowed =
        ui.is_enabled() && ui.ctx().memory(|m| m.allows_interaction(ui.layer_id()));
    if clicked.is_none() && shortcuts_allowed {
        for (i, item) in items.iter().enumerate() {
            if !item.enabled {
                continue;
            }
            if let Some(sc) = item.shortcut
                && ui.input_mut(|input| input.consume_shortcut(&sc))
            {
                clicked = Some(i);
                break;
            }
        }
    }

    ui.allocate_rect(bar_rect, Sense::hover());
    clicked
}

// Reference logic, ported verbatim.
fn fit_fkey_items(area_width: f32, key_widths: &[f32], longest_label_w: f32, inset: f32) -> usize {
    let mut n = key_widths.len();
    while n > 0 {
        let slot_w = area_width / n as f32;
        let widest_key = key_widths[..n].iter().cloned().fold(0.0f32, f32::max);
        if slot_w >= widest_key + longest_label_w + 3.0 * inset {
            break;
        }
        n -= 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use egui::RawInput;

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    #[test]
    fn fit_fkey_items_full_width_fits_all() {
        let widths = [30.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 30.0, 10.0];
        assert_eq!(fit_fkey_items(2000.0, &widths, 60.0, 4.0), 9);
    }

    #[test]
    fn fit_fkey_items_narrow_width_drops_slots() {
        let widths = [30.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 30.0, 10.0];
        assert_eq!(fit_fkey_items(400.0, &widths, 60.0, 4.0), 3);
    }

    #[test]
    fn fit_fkey_items_zero_width_yields_zero() {
        let widths = [30.0, 10.0];
        assert_eq!(fit_fkey_items(0.0, &widths, 60.0, 4.0), 0);
    }

    #[test]
    fn fkey_bar_smoke_test_returns_none_without_input() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let items: Vec<FKey<'_>> = (1..=10)
            .map(|i| {
                let key = match i {
                    1 => "F1",
                    2 => "F2",
                    3 => "F3",
                    4 => "F4",
                    5 => "F5",
                    6 => "F6",
                    7 => "F7",
                    8 => "F8",
                    9 => "F9",
                    _ => "F10",
                };
                FKey::new(key, "Label")
            })
            .collect();

        let mut clicked = None;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            clicked = fkey_bar(ui, &items);
        });
        // Laying out text rasterizes glyphs into the font atlas; the resulting
        // texture delta must be consumed (normally by a renderer) or applied.
        output.textures_delta.clear();

        assert_eq!(clicked, None);
    }

    fn key_event(key: egui::Key) -> RawInput {
        RawInput {
            events: vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        }
    }

    fn f_items() -> Vec<FKey<'static>> {
        (1..=10)
            .map(|i| {
                let (key, egui_key) = match i {
                    1 => ("F1", egui::Key::F1),
                    2 => ("F2", egui::Key::F2),
                    3 => ("F3", egui::Key::F3),
                    4 => ("F4", egui::Key::F4),
                    5 => ("F5", egui::Key::F5),
                    6 => ("F6", egui::Key::F6),
                    7 => ("F7", egui::Key::F7),
                    8 => ("F8", egui::Key::F8),
                    9 => ("F9", egui::Key::F9),
                    _ => ("F10", egui::Key::F10),
                };
                FKey::new(key, "Label").key_shortcut(egui_key)
            })
            .collect()
    }

    #[test]
    fn fkey_shortcut_returns_index() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());
        let items = f_items();

        let mut clicked = None;
        let mut output = ctx.run_ui(key_event(egui::Key::F2), |ui| {
            clicked = fkey_bar(ui, &items);
        });
        output.textures_delta.clear();

        assert_eq!(clicked, Some(1));
    }

    #[test]
    fn fkey_disabled_shortcut_ignored() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());
        let mut items = f_items();
        items[1] = items[1].enabled(false);

        let mut clicked = None;
        let mut output = ctx.run_ui(key_event(egui::Key::F2), |ui| {
            clicked = fkey_bar(ui, &items);
        });
        output.textures_delta.clear();

        assert_eq!(clicked, None);
    }

    #[test]
    fn fkey_shortcut_works_for_dropped_slot() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());
        let items = f_items();

        let mut clicked = None;
        let mut output = ctx.run_ui(key_event(egui::Key::F2), |ui| {
            ui.allocate_ui(vec2(40.0, 60.0), |ui| {
                clicked = fkey_bar(ui, &items);
            });
        });
        output.textures_delta.clear();

        assert_eq!(clicked, Some(1));
    }

    #[test]
    fn fkey_shortcut_ignored_under_modal() {
        use crate::widgets::Dialog;

        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());
        let items = f_items();

        // Warm-up frames: the modal-layer gate reads the previous pass's
        // bookkeeping, so it only takes effect from the second frame the
        // dialog is shown.
        let run_frame = |ctx: &egui::Context, input: RawInput, clicked: &mut Option<usize>| {
            let mut output = ctx.run_ui(input, |ui| {
                *clicked = fkey_bar(ui, &items);
                Dialog::new(egui::Id::new("modal"), "T").show(ui.ctx(), |ui| {
                    ui.label("hi");
                });
            });
            output.textures_delta.clear();
        };

        let mut clicked = None;
        for _ in 0..2 {
            run_frame(&ctx, RawInput::default(), &mut clicked);
        }
        run_frame(&ctx, key_event(egui::Key::F2), &mut clicked);

        assert_eq!(clicked, None);
    }

    #[test]
    fn fkey_const_builders() {
        const ITEMS: [FKey<'static>; 2] = [
            FKey::new("F1", "Help").key_shortcut(egui::Key::F1),
            FKey::new("F2", "Open"),
        ];
        assert!(ITEMS[0].enabled);
        assert!(ITEMS[0].shortcut.is_some());
        assert!(ITEMS[1].shortcut.is_none());
    }
}
