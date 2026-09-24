//! 1-dot frame painter shared by titled panels and other bordered widgets.

use egui::{Color32, Painter, Rect, Stroke, StrokeKind};

use crate::style::dot;

/// Paints a 1-dot frame just inside `rect`.
pub fn paint_frame(painter: &Painter, rect: Rect, color: Color32) {
    painter.rect_stroke(
        rect,
        0,
        Stroke::new(dot(painter.ctx()), color),
        StrokeKind::Inside,
    );
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
}
