//! Titled PC-98 pane: cyan title strip, 1-dot frame, clipped content area.

use egui::{Align, InnerResponse, Layout, Rect, Sense, Ui, UiBuilder, pos2, vec2};

use crate::style::{dots, palette};
use crate::text::cell_width;
use crate::widgets::paint_frame;

/// Titled PC-98 pane with an optional right-aligned title-strip slot.
pub struct TitledPanel<'a> {
    title: &'a str,
    focused: bool,
    double: bool,
}

impl<'a> TitledPanel<'a> {
    /// New panel titled `title`. Focused by default.
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            focused: true,
            double: false,
        }
    }

    /// Unfocused panels draw the title strip and frame in `dim`.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Draws a double-line frame instead of the single 1-dot frame.
    pub fn double_frame(mut self, double: bool) -> Self {
        self.double = double;
        self
    }

    /// Shows the panel, filling the available space in `ui`.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        self.show_with_title(ui, |_| {}, add_contents)
    }

    /// Shows the panel with an extra right-aligned slot in the title strip.
    pub fn show_with_title<R>(
        self,
        ui: &mut Ui,
        title_right: impl FnOnce(&mut Ui),
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let palette = palette(ui.ctx());
        let font_id = egui::TextStyle::Body.resolve(ui.style());
        let (strip_color, frame_color) = if self.focused {
            (palette.title_bg, palette.frame)
        } else {
            (palette.dim, palette.dim)
        };

        let outer = ui.available_rect_before_wrap();
        let painter = ui.painter();

        let title_galley = painter.layout_no_wrap(self.title.to_owned(), font_id, palette.title_fg);
        let title_h = title_galley.size().y + dots(ui.ctx(), 4.0);
        let title_rect = Rect::from_min_size(outer.min, vec2(outer.width(), title_h));

        painter.rect_filled(outer, 0, palette.ground);
        painter.rect_filled(title_rect, 0, strip_color);
        let text_pos = pos2(
            title_rect.min.x + dots(ui.ctx(), 8.0),
            title_rect.center().y - title_galley.size().y / 2.0,
        );
        let title_text_right = text_pos.x + title_galley.size().x;
        painter.galley(text_pos, title_galley, palette.title_fg);

        let slot_min_x = title_text_right + cell_width(ui);
        let slot_max_x = title_rect.max.x - dots(ui.ctx(), 8.0);
        if slot_max_x > slot_min_x {
            let slot = Rect::from_min_max(
                pos2(slot_min_x, title_rect.min.y),
                pos2(slot_max_x, title_rect.max.y),
            );
            let mut child = ui.new_child(
                UiBuilder::new()
                    .max_rect(slot)
                    .layout(Layout::right_to_left(Align::Center)),
            );
            child.set_clip_rect(slot);
            child.visuals_mut().override_text_color = Some(palette.title_fg);
            title_right(&mut child);
        }

        // Allocated here, before the content, so content widgets (added next,
        // hence on top) keep priority for clicks; this response only fires on
        // parts of the panel the content leaves uncovered.
        let response = ui.allocate_rect(outer, Sense::click());

        let content_rect = Rect::from_min_max(outer.min + vec2(0.0, title_h), outer.max)
            .shrink(dots(ui.ctx(), 4.0));
        let mut content_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(content_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        content_ui.set_clip_rect(content_rect);
        let inner = add_contents(&mut content_ui);

        // Drawn last so the border stays on top of the content.
        paint_frame(ui.painter(), outer, frame_color);
        if self.double {
            // Inner top edge sits on the title strip's bottom edge, not 2 dots
            // below the outer top, so it never crosses the title text.
            let d2 = dots(ui.ctx(), 2.0);
            let inner = Rect::from_min_max(
                pos2(outer.min.x + d2, title_rect.max.y),
                pos2(outer.max.x - d2, outer.max.y - d2),
            );
            paint_frame(ui.painter(), inner, frame_color);
        }

        InnerResponse { inner, response }
    }
}

/// PC-98 style pane: cyan title strip, 1-dot frame. Fills all available space in `ui`;
/// wrap it in `ui.allocate_ui(size, ..)` to constrain it.
pub fn panel<R>(
    ui: &mut Ui,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    TitledPanel::new(title).show(ui, add_contents)
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

    #[test]
    fn titled_panel_title_slot_runs() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut slot_ran = false;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            TitledPanel::new("T").show_with_title(
                ui,
                |_ui| slot_ran = true,
                |ui| {
                    ui.label("x");
                },
            );
        });
        output.textures_delta.clear();

        assert!(slot_ran);
    }

    #[test]
    fn titled_panel_unfocused_smoke() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut rect = Rect::NOTHING;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let response = TitledPanel::new("T").focused(false).show(ui, |ui| {
                ui.label("x");
            });
            rect = response.response.rect;
        });
        output.textures_delta.clear();

        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
    }

    #[test]
    fn title_slot_skipped_when_no_room() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut slot_ran = false;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            ui.allocate_ui(vec2(20.0, 100.0), |ui| {
                TitledPanel::new("A very long title that leaves no room for a slot")
                    .show_with_title(
                        ui,
                        |_ui| slot_ran = true,
                        |ui| {
                            ui.label("x");
                        },
                    );
            });
        });
        output.textures_delta.clear();

        assert!(!slot_ran);
    }

    #[test]
    fn double_frame_panel_smoke_test() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            TitledPanel::new("T").double_frame(true).show(ui, |ui| {
                ui.label("x");
            });
        });
        output.textures_delta.clear();
    }
}
