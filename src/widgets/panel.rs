//! Titled PC-98 pane: cyan title strip, 1-dot frame, clipped content area.

use egui::{Align, InnerResponse, Layout, Rect, Sense, Ui, UiBuilder, pos2, vec2};

use crate::style::{dots, palette};
use crate::widgets::paint_frame;

/// PC-98 style pane: cyan title strip, 1-dot frame. Fills all available space in `ui`;
/// wrap it in `ui.allocate_ui(size, ..)` to constrain it.
pub fn panel<R>(
    ui: &mut Ui,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let palette = palette(ui.ctx());
    let font_id = egui::TextStyle::Body.resolve(ui.style());

    let outer = ui.available_rect_before_wrap();
    let painter = ui.painter();

    let title_galley = painter.layout_no_wrap(title.to_owned(), font_id, palette.title_fg);
    let title_h = title_galley.size().y + dots(ui.ctx(), 4.0);
    let title_rect = Rect::from_min_size(outer.min, vec2(outer.width(), title_h));

    painter.rect_filled(outer, 0, palette.ground);
    painter.rect_filled(title_rect, 0, palette.title_bg);
    let text_pos = pos2(
        title_rect.min.x + dots(ui.ctx(), 8.0),
        title_rect.center().y - title_galley.size().y / 2.0,
    );
    painter.galley(text_pos, title_galley, palette.title_fg);

    let content_rect =
        Rect::from_min_max(outer.min + vec2(0.0, title_h), outer.max).shrink(dots(ui.ctx(), 4.0));
    let mut content_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(content_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    content_ui.set_clip_rect(content_rect);
    let inner = add_contents(&mut content_ui);

    // Drawn last so the border stays on top of the content.
    paint_frame(ui.painter(), outer, palette.frame);

    let response = ui.allocate_rect(outer, Sense::hover());
    InnerResponse { inner, response }
}

#[cfg(test)]
mod tests {
    use egui::RawInput;

    use crate::style::apply_with;
    use crate::{Palette, widgets::header_bar};

    use super::*;

    #[test]
    fn panel_and_header_bar_smoke_test() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut panel_rect = Rect::NOTHING;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let response = panel(ui, "T", |ui| {
                ui.label("x");
            });
            panel_rect = response.response.rect;
            header_bar(ui, "L", Some("R"));
        });
        // Laying out text rasterizes glyphs into the font atlas; the resulting
        // texture delta must be consumed (normally by a renderer) or applied.
        output.textures_delta.clear();

        assert!(panel_rect.width() > 0.0);
        assert!(panel_rect.height() > 0.0);
    }
}
