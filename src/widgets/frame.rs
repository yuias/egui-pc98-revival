//! 1-dot frame painter shared by titled panels and other bordered widgets.

use egui::{Color32, Painter, Rect, Stroke, StrokeKind};

use crate::style::{dot, dots};

/// Paints a 1-dot frame just inside `rect`.
pub fn paint_frame(painter: &Painter, rect: Rect, color: Color32) {
    painter.rect_stroke(
        rect,
        0,
        Stroke::new(dot(painter.ctx()), color),
        StrokeKind::Inside,
    );
}

/// Paints a double 1-dot frame inside `rect`: an outer line, a 1-dot gap, and an inner line.
pub fn paint_double_frame(painter: &Painter, rect: Rect, color: Color32) {
    paint_frame(painter, rect, color);
    let ctx = painter.ctx();
    // A smaller inner rect would collapse or invert, so only the outer frame is drawn.
    if rect.width() < dots(ctx, 6.0) || rect.height() < dots(ctx, 6.0) {
        return;
    }
    paint_frame(painter, rect.shrink(dots(ctx, 2.0)), color);
}

#[cfg(test)]
mod tests {
    use egui::{RawInput, pos2, vec2};

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    #[test]
    fn paint_frame_smoke_test() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let painter = ui.painter();
            let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(16.0, 16.0));
            paint_frame(painter, rect, Color32::WHITE);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn paint_double_frame_smoke_test() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let painter = ui.painter();
            let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(16.0, 16.0));
            paint_double_frame(painter, rect, Color32::WHITE);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn paint_double_frame_small_rect_does_not_panic() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let painter = ui.painter();
            let small = Rect::from_min_size(pos2(0.0, 0.0), vec2(4.0, 4.0));
            paint_double_frame(painter, small, Color32::WHITE);
            let zero = Rect::from_min_size(pos2(0.0, 0.0), vec2(0.0, 0.0));
            paint_double_frame(painter, zero, Color32::WHITE);
        });
        output.textures_delta.clear();
    }
}
