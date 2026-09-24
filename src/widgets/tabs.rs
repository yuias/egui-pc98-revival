//! Row of tabs: the title strip of a `TitledPanel`, or a standalone bar.

use egui::{Rect, Response, Sense, TextStyle, Ui, pos2, vec2};

use crate::style::{dots, palette};
use crate::text::{cell_width, cells};
use crate::widgets::paint_frame;

/// Where a [`tab_strip`] is drawn, which picks its selected/hovered colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabStyle {
    /// For the title strip of a `TitledPanel`: current tab cut out in `ground`.
    TitleStrip,
    /// For use on `ground`.
    Bar,
}

/// Row of tabs; clicking a tab sets `*selected`. `response.changed()` is true
/// on the frame the selection changes.
pub fn tab_strip(ui: &mut Ui, selected: &mut usize, labels: &[&str], style: TabStyle) -> Response {
    let palette = palette(ui.ctx());
    let ctx = ui.ctx().clone();
    let cell_w = cell_width(ui);
    let font_id = TextStyle::Body.resolve(ui.style());
    let gap = dots(&ctx, 4.0);
    let tab_h = ui.text_style_height(&TextStyle::Body) + dots(&ctx, 2.0);

    let widths: Vec<f32> = labels
        .iter()
        .map(|label| (cells(label) + 2) as f32 * cell_w)
        .collect();
    let total_w: f32 = widths.iter().sum::<f32>() + gap * labels.len().saturating_sub(1) as f32;

    let (strip_rect, mut strip_response) =
        ui.allocate_exact_size(vec2(total_w, tab_h), Sense::hover());

    let mut tab_rects = Vec::with_capacity(labels.len());
    let mut x = strip_rect.min.x;
    for &w in &widths {
        tab_rects.push(Rect::from_min_size(
            pos2(x, strip_rect.min.y),
            vec2(w, tab_h),
        ));
        x += w + gap;
    }

    // Interact first so every tab's click is resolved before any is painted:
    // a click can change which tab is "selected" for the paint pass below.
    let original_selected = *selected;
    let mut hovered = vec![false; labels.len()];
    for (i, rect) in tab_rects.iter().enumerate() {
        let response = ui.interact(*rect, strip_response.id.with(i), Sense::click());
        hovered[i] = response.hovered();
        if response.clicked() {
            *selected = i;
        }
    }
    if *selected != original_selected {
        strip_response.mark_changed();
    }

    let painter = ui.painter();
    for (i, (rect, label)) in tab_rects.iter().zip(labels.iter()).enumerate() {
        let is_selected = *selected == i;
        let is_hovered = !is_selected && hovered[i];

        let (fill, text_color, frame_color) = match (style, is_selected, is_hovered) {
            (TabStyle::TitleStrip, true, _) => (Some(palette.ground), palette.title_bg, None),
            (TabStyle::TitleStrip, false, true) => (Some(palette.accent), palette.hover_fg, None),
            (TabStyle::TitleStrip, false, false) => {
                (None, palette.title_fg, Some(palette.title_fg))
            }
            (TabStyle::Bar, true, _) => (Some(palette.selected_bg), palette.selected_fg, None),
            (TabStyle::Bar, false, true) => (Some(palette.accent), palette.hover_fg, None),
            (TabStyle::Bar, false, false) => (None, palette.text, Some(palette.dim)),
        };

        if let Some(fill) = fill {
            painter.rect_filled(*rect, 0, fill);
        }
        if let Some(frame_color) = frame_color {
            paint_frame(painter, *rect, frame_color);
        }

        let galley = painter.layout_no_wrap((*label).to_owned(), font_id.clone(), text_color);
        let text_pos = rect.center() - galley.size() / 2.0;
        painter.galley(text_pos, galley, text_color);
    }

    strip_response
}

#[cfg(test)]
mod tests {
    use egui::{Event, Modifiers, PointerButton, RawInput};

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    #[test]
    fn tab_strip_without_input_keeps_selection() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut selected = 0usize;
        let mut changed = false;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let response = tab_strip(ui, &mut selected, &["A:", "B:", "C:"], TabStyle::Bar);
            changed = response.changed();
        });
        output.textures_delta.clear();

        assert_eq!(selected, 0);
        assert!(!changed);
    }

    #[test]
    fn tab_strip_click_selects() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut selected = 0usize;

        // Frame 1: no input, just record the strip rect.
        let mut strip_rect = Rect::NOTHING;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            strip_rect = tab_strip(ui, &mut selected, &["A:", "B:", "C:"], TabStyle::Bar).rect;
        });
        output.textures_delta.clear();

        // Tab 0 is `(cells("A:") + 2) = 4` cells wide; the click point sits
        // inside tab 1, the second tab, well clear of the frame's 1-dot edge.
        let mut cell_w = 0.0;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            cell_w = cell_width(ui);
        });
        output.textures_delta.clear();

        // egui's hit-testing looks at widget rects registered two passes ago,
        // so the strip must be drawn at a stable position for two frames
        // before pointer events against it are picked up.
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            tab_strip(ui, &mut selected, &["A:", "B:", "C:"], TabStyle::Bar);
        });
        output.textures_delta.clear();

        let tab0_w = 4.0 * cell_w;
        let gap = dots(&ctx, 4.0);
        let tab1_center_x = strip_rect.min.x + tab0_w + gap + 2.0 * cell_w;
        let pos = egui::pos2(tab1_center_x, strip_rect.center().y);

        // Frame: press.
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
                let response = tab_strip(ui, &mut selected, &["A:", "B:", "C:"], TabStyle::Bar);
                changed = response.changed();
            },
        );
        output.textures_delta.clear();

        // Frame: release, which is what egui counts as a click.
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
                let response = tab_strip(ui, &mut selected, &["A:", "B:", "C:"], TabStyle::Bar);
                changed = response.changed();
            },
        );
        output.textures_delta.clear();

        assert_eq!(selected, 1);
        assert!(changed);
    }

    #[test]
    fn tab_strip_out_of_range_selected_is_untouched() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut selected = 99usize;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            tab_strip(ui, &mut selected, &["A:", "B:", "C:"], TabStyle::Bar);
        });
        output.textures_delta.clear();

        assert_eq!(selected, 99);
    }
}
