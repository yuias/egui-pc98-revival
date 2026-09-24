//! Two-column key/description help list, e.g. for a HELP dialog.

use egui::{Response, Sense, TextStyle, Ui, Vec2, pos2, vec2};

use crate::style::palette;
use crate::text::{cell_width, cells, truncate_tail};

/// Two-column help rows: key right-aligned in `accent`, description in `text`.
pub fn key_help(ui: &mut Ui, rows: &[(&str, &str)]) -> Response {
    if rows.is_empty() {
        return ui.allocate_exact_size(Vec2::ZERO, Sense::hover()).1;
    }

    let pal = palette(ui.ctx());
    let cell_w = cell_width(ui);
    let row_h = ui.text_style_height(&TextStyle::Body);
    let font_id = TextStyle::Body.resolve(ui.style());

    let key_col_cells = rows.iter().map(|(key, _)| cells(key)).max().unwrap_or(0);
    let key_col_w = key_col_cells as f32 * cell_w;

    let mut response: Option<Response> = None;
    for (key, desc) in rows.iter().copied() {
        let (rect, row_response) =
            ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::hover());

        let key_col_right = rect.left() + key_col_w;
        let desc_x = key_col_right + 2.0 * cell_w;
        let avail_cells = ((rect.right() - desc_x) / cell_w).floor().max(0.0) as usize;
        let desc = truncate_tail(desc, avail_cells);

        let painter = ui.painter();

        let key_galley = painter.layout_no_wrap(key.to_owned(), font_id.clone(), pal.accent);
        let key_pos = pos2(
            key_col_right - key_galley.size().x,
            rect.center().y - key_galley.size().y / 2.0,
        );
        painter.galley(key_pos, key_galley, pal.accent);

        let desc_galley = painter.layout_no_wrap(desc, font_id.clone(), pal.text);
        let desc_pos = pos2(desc_x, rect.center().y - desc_galley.size().y / 2.0);
        painter.galley(desc_pos, desc_galley, pal.text);

        response = Some(match response {
            Some(prev) => prev.union(row_response),
            None => row_response,
        });
    }

    response.expect("rows is non-empty")
}

#[cfg(test)]
mod tests {
    use egui::RawInput;

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    #[test]
    fn key_help_smoke() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let rows = [
            ("F1", "Show this help"),
            ("Esc", "Close"),
            ("Space", "Toggle"),
        ];

        let mut height = 0.0;
        let mut row_h = 0.0;
        let mut spacing_y = 0.0;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            row_h = ui.text_style_height(&TextStyle::Body);
            spacing_y = ui.spacing().item_spacing.y;
            height = key_help(ui, &rows).rect.height();
        });
        output.textures_delta.clear();

        let expected = 3.0 * row_h + 2.0 * spacing_y;
        assert!(
            (height - expected).abs() < 1e-3,
            "height {height}, expected {expected}"
        );
    }

    #[test]
    fn key_help_empty_rows() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut response = None;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            response = Some(key_help(ui, &[]));
        });
        output.textures_delta.clear();

        let response = response.unwrap();
        assert_eq!(response.rect.size(), Vec2::ZERO);
    }
}
