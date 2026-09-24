//! Function-key bar: a row of key/label slots, PC-98 style.

use egui::{Rect, Sense, Ui, pos2, vec2};

use crate::style::{dot, dots, palette};

/// One function-key slot: the key label (e.g. `"F1"`) and its action label.
#[derive(Clone, Copy, Debug)]
pub struct FKey<'a> {
    pub key: &'a str,
    pub label: &'a str,
}

/// Function-key bar. Returns the index of the slot clicked this frame.
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
        .map(|item| painter.layout_no_wrap(item.label.to_owned(), font_id.clone(), palette.fkey_fg))
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
            let response = ui.interact(slot, ui.id().with(("pc98_fkey", i)), Sense::click());
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
            let label_bg = if response.hovered() {
                palette.accent
            } else {
                palette.fkey_bg
            };
            painter.rect_filled(label_rect, 0, label_bg);

            let label_galley = &label_galleys[i];
            let label_pos = label_rect.center() - label_galley.size() / 2.0;
            painter.galley(label_pos, label_galley.clone(), palette.fkey_fg);
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
            .map(|i| FKey {
                key: match i {
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
                },
                label: "Label",
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
}
