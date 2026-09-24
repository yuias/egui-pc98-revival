//! `[M]`-style letter toggle box.

use egui::{Color32, Response, Sense, TextStyle, Ui, vec2};

use crate::style::{palette, snap_rect};
use crate::text::{cell_width, cells};
use crate::widgets::paint_frame;

/// `[M]`-style letter box: on = filled `danger` with `ground` letter; off = dim outline and letter.
pub fn toggle_box(ui: &mut Ui, on: &mut bool, letter: &str) -> Response {
    let color = palette(ui.ctx()).danger;
    toggle_box_colored(ui, on, letter, color)
}

/// `toggle_box` with a custom "on" fill color.
pub fn toggle_box_colored(ui: &mut Ui, on: &mut bool, letter: &str, color: Color32) -> Response {
    let ctx = ui.ctx().clone();
    let pal = palette(&ctx);
    let cell_w = cell_width(ui);
    let size = vec2(
        (cells(letter) as f32 + 1.0) * cell_w,
        ui.text_style_height(&TextStyle::Body),
    );
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());

    // egui gives a focused `Sense::click()` widget a synthetic click on
    // Space/Enter (`Flags::FAKE_PRIMARY_CLICKED`), so `clicked()` already
    // covers both pointer and keyboard activation.
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    let rect = snap_rect(&ctx, rect);
    let painter = ui.painter();
    let fg = if *on {
        painter.rect_filled(rect, 0, color);
        pal.ground
    } else {
        paint_frame(painter, rect, pal.dim);
        pal.dim
    };

    if ui.is_enabled() && (response.hovered() || response.has_focus()) {
        paint_frame(painter, rect, pal.accent);
    }

    let font_id = TextStyle::Body.resolve(ui.style());
    let galley = painter.layout_no_wrap(letter.to_owned(), font_id, fg);
    let pos = rect.center() - galley.size() / 2.0;
    painter.galley(pos, galley, fg);

    response
}

#[cfg(test)]
mod tests {
    use egui::{Event, Key, Modifiers, PointerButton, RawInput, Rect};

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    #[test]
    fn toggle_box_no_input_keeps_state() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut on = false;
        let mut changed = false;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let response = toggle_box(ui, &mut on, "M");
            changed = response.changed();
        });
        output.textures_delta.clear();

        assert!(!on);
        assert!(!changed);
    }

    #[test]
    fn toggle_box_click_toggles() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut on = false;

        // Warm-up frames: egui's hit-testing uses widget rects registered two
        // passes ago, so the box must be drawn at a stable position first.
        let mut rect = Rect::NOTHING;
        for _ in 0..2 {
            let mut output = ctx.run_ui(RawInput::default(), |ui| {
                rect = toggle_box(ui, &mut on, "M").rect;
            });
            output.textures_delta.clear();
        }

        let pos = rect.center();

        let mut changed = false;
        let mut output = ctx.run_ui(
            RawInput {
                events: vec![
                    Event::PointerMoved(pos),
                    Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    },
                ],
                ..Default::default()
            },
            |ui| {
                let response = toggle_box(ui, &mut on, "M");
                changed = response.changed();
            },
        );
        output.textures_delta.clear();

        let mut output = ctx.run_ui(
            RawInput {
                events: vec![Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                }],
                ..Default::default()
            },
            |ui| {
                let response = toggle_box(ui, &mut on, "M");
                changed = response.changed();
            },
        );
        output.textures_delta.clear();

        assert!(on);
        assert!(changed);
    }

    #[test]
    fn toggle_box_space_toggles_when_focused() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut on = false;

        let mut id = None;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            id = Some(toggle_box(ui, &mut on, "M").id);
        });
        output.textures_delta.clear();
        let id = id.unwrap();
        ctx.memory_mut(|m| m.request_focus(id));

        // Warm-up frame: focus must already be set on the *previous* pass for
        // the widget to treat Space as activation rather than a focus change.
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            toggle_box(ui, &mut on, "M");
        });
        output.textures_delta.clear();

        let mut output = ctx.run_ui(
            RawInput {
                events: vec![Event::Key {
                    key: Key::Space,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                }],
                ..Default::default()
            },
            |ui| {
                toggle_box(ui, &mut on, "M");
            },
        );
        output.textures_delta.clear();

        assert!(on);
    }
}
