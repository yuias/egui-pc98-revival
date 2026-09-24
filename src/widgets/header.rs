//! Full-width header bar.

use egui::{Rect, Response, Sense, Ui, pos2, vec2};

use crate::style::{dots, palette};

/// Full-width solid bar (`BAR_BG`) with `BAR_FG` text; `right` is right-aligned.
pub fn header_bar(ui: &mut Ui, left: &str, right: Option<&str>) -> Response {
    let palette = palette(ui.ctx());
    let font_id = egui::TextStyle::Body.resolve(ui.style());

    let outer = ui.available_rect_before_wrap();
    let painter = ui.painter();

    let left_galley = painter.layout_no_wrap(left.to_owned(), font_id.clone(), palette.bar_fg);
    let height = left_galley.size().y + dots(ui.ctx(), 4.0);
    let bar_rect = Rect::from_min_size(outer.min, vec2(ui.available_width(), height));

    painter.rect_filled(bar_rect, 0, palette.bar_bg);

    let left_pos = pos2(
        bar_rect.min.x + dots(ui.ctx(), 8.0),
        bar_rect.center().y - left_galley.size().y / 2.0,
    );
    painter.galley(left_pos, left_galley, palette.bar_fg);

    if let Some(right) = right {
        let right_galley = painter.layout_no_wrap(right.to_owned(), font_id, palette.bar_fg);
        let right_pos = pos2(
            bar_rect.max.x - dots(ui.ctx(), 8.0) - right_galley.size().x,
            bar_rect.center().y - right_galley.size().y / 2.0,
        );
        painter.galley(right_pos, right_galley, palette.bar_fg);
    }

    ui.allocate_rect(bar_rect, Sense::hover())
}
