//! Text-drawn checkbox `[*]` and radio `(*)`, keyboard-focusable like the
//! stock egui widgets.

use egui::{Color32, Painter, Response, Sense, TextStyle, Ui, WidgetType, pos2, vec2};

use crate::style::{palette, snap_rect};
use crate::text::{cell_width, cells};

/// Checkbox drawn with the font: `[*] label` / `[ ] label`. Space toggles when focused.
pub fn text_checkbox(ui: &mut Ui, checked: &mut bool, label: &str) -> Response {
    let mut response = text_toggle(ui, ["[", "]"], *checked, label, WidgetType::Checkbox);
    // egui gives a focused `Sense::click()` widget a synthetic click on
    // Space/Enter (`Flags::FAKE_PRIMARY_CLICKED`), so `clicked()` already
    // covers both pointer and keyboard activation.
    if response.clicked() {
        *checked = !*checked;
        response.mark_changed();
    }
    response
}

/// Radio drawn with the font: `(*) label` / `( ) label`. Selects `value` on click or Space.
pub fn text_radio<T: PartialEq>(ui: &mut Ui, current: &mut T, value: T, label: &str) -> Response {
    let selected = *current == value;
    let mut response = text_toggle(ui, ["(", ")"], selected, label, WidgetType::RadioButton);
    if response.clicked() && *current != value {
        *current = value;
        response.mark_changed();
    }
    response
}

/// Shared painter for `text_checkbox` / `text_radio`: `marker[0]` + (`*`/` `) +
/// `marker[1]` + a space + `label`, each part colored independently so the
/// marker glyph can stay `accent` while brackets and label are `text`.
fn text_toggle(
    ui: &mut Ui,
    marker: [&str; 2],
    on: bool,
    label: &str,
    kind: WidgetType,
) -> Response {
    let ctx = ui.ctx().clone();
    let pal = palette(&ctx);
    let cell_w = cell_width(ui);
    let width = if label.is_empty() {
        3.0 * cell_w
    } else {
        (4.0 + cells(label) as f32) * cell_w
    };
    let size = vec2(width, ui.text_style_height(&TextStyle::Body));
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    let enabled = ui.is_enabled();
    let active = enabled && (response.hovered() || response.has_focus());

    let rect = snap_rect(&ctx, rect);
    let painter = ui.painter();
    if active {
        painter.rect_filled(rect, 0, pal.accent);
    }

    let (bracket_color, marker_color) = if !enabled {
        (pal.dim, pal.dim)
    } else if active {
        (pal.hover_fg, pal.hover_fg)
    } else {
        (pal.text, pal.accent)
    };

    let font_id = TextStyle::Body.resolve(ui.style());
    let paint_cell = |painter: &Painter, col: f32, text: &str, color: Color32| {
        let galley = painter.layout_no_wrap(text.to_owned(), font_id.clone(), color);
        let pos = pos2(
            rect.left() + col * cell_w,
            rect.center().y - galley.size().y / 2.0,
        );
        painter.galley(pos, galley, color);
    };

    paint_cell(painter, 0.0, marker[0], bracket_color);
    paint_cell(painter, 1.0, if on { "*" } else { " " }, marker_color);
    paint_cell(painter, 2.0, marker[1], bracket_color);
    if !label.is_empty() {
        paint_cell(painter, 4.0, label, bracket_color);
    }

    response.widget_info(|| egui::WidgetInfo::selected(kind, enabled, on, label));

    response
}

#[cfg(test)]
mod tests {
    use egui::{Event, Key, Modifiers, PointerButton, RawInput, Rect};

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    #[test]
    fn text_checkbox_click_toggles() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut checked = false;

        // Warm-up frames: hit-testing uses widget rects registered two
        // passes ago, so the box must be drawn at a stable position first.
        let mut rect = Rect::NOTHING;
        for _ in 0..2 {
            let mut output = ctx.run_ui(RawInput::default(), |ui| {
                rect = text_checkbox(ui, &mut checked, "Loop").rect;
            });
            output.textures_delta.clear();
        }

        let pos = rect.center();

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
                text_checkbox(ui, &mut checked, "Loop");
            },
        );
        output.textures_delta.clear();

        let mut changed = false;
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
                let response = text_checkbox(ui, &mut checked, "Loop");
                changed = response.changed();
            },
        );
        output.textures_delta.clear();

        assert!(checked);
        assert!(changed);
    }

    #[test]
    fn text_checkbox_space_toggles_when_focused() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut checked = false;

        let mut id = None;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            id = Some(text_checkbox(ui, &mut checked, "Loop").id);
        });
        output.textures_delta.clear();
        let id = id.unwrap();
        ctx.memory_mut(|m| m.request_focus(id));

        // Warm-up frame: focus must already be set on the *previous* pass for
        // the widget to treat Space as activation rather than a focus change.
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            text_checkbox(ui, &mut checked, "Loop");
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
                text_checkbox(ui, &mut checked, "Loop");
            },
        );
        output.textures_delta.clear();

        assert!(checked);
    }

    #[test]
    fn text_radio_click_selects() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut current = 0;

        let mut rect = Rect::NOTHING;
        for _ in 0..2 {
            let mut output = ctx.run_ui(RawInput::default(), |ui| {
                rect = text_radio(ui, &mut current, 1, "B:").rect;
            });
            output.textures_delta.clear();
        }

        let pos = rect.center();

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
                text_radio(ui, &mut current, 1, "B:");
            },
        );
        output.textures_delta.clear();

        let mut changed = false;
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
                let response = text_radio(ui, &mut current, 1, "B:");
                changed = response.changed();
            },
        );
        output.textures_delta.clear();

        assert_eq!(current, 1);
        assert!(changed);
    }

    #[test]
    fn text_radio_click_on_selected_is_unchanged() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut current = 1;

        let mut rect = Rect::NOTHING;
        for _ in 0..2 {
            let mut output = ctx.run_ui(RawInput::default(), |ui| {
                rect = text_radio(ui, &mut current, 1, "B:").rect;
            });
            output.textures_delta.clear();
        }

        let pos = rect.center();

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
                text_radio(ui, &mut current, 1, "B:");
            },
        );
        output.textures_delta.clear();

        let mut changed = false;
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
                let response = text_radio(ui, &mut current, 1, "B:");
                changed = response.changed();
            },
        );
        output.textures_delta.clear();

        assert_eq!(current, 1);
        assert!(!changed);
    }

    #[test]
    fn text_checkbox_empty_label_width() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut checked = false;
        let mut width = 0.0;
        let mut cell_w = 0.0;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            width = text_checkbox(ui, &mut checked, "").rect.width();
            cell_w = cell_width(ui);
        });
        output.textures_delta.clear();

        assert!((width - 3.0 * cell_w).abs() < 0.5);
    }
}
