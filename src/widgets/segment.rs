//! Row or column of equal blocks with 1-dot gaps: level meter, volume
//! blocks, gauge.

use egui::{Color32, Pos2, Rect, Response, Sense, TextStyle, Ui, Vec2, pos2, vec2};

use crate::style::{dot, dots, palette, snap_rect};
use crate::text::cell_width;

/// Row or column of equal blocks with 1-dot gaps (level meter, volume blocks, gauge).
#[derive(Clone, Copy, Debug)]
pub struct SegmentBar {
    segments: usize,
    value: f32,
    fill: Option<Color32>,
    warn: Option<(usize, Color32)>,
    vertical: bool,
    desired_size: Option<Vec2>,
}

impl SegmentBar {
    /// `segments` blocks, lit up to `value` (clamped to `0..=1`).
    pub fn new(segments: usize, value: f32) -> Self {
        Self {
            segments,
            value,
            fill: None,
            warn: None,
            vertical: false,
            desired_size: None,
        }
    }

    /// Color of lit segments below the `warn` threshold. Defaults to `palette.frame`.
    pub fn fill(mut self, color: Color32) -> Self {
        self.fill = Some(color);
        self
    }

    /// Lit segments with index `>= from` use `color` instead of `fill`.
    pub fn warn(mut self, from: usize, color: Color32) -> Self {
        self.warn = Some((from, color));
        self
    }

    /// Stack segments vertically, index 0 at the bottom.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Overrides the default size.
    pub fn desired_size(mut self, size: Vec2) -> Self {
        self.desired_size = Some(size);
        self
    }

    /// Display only (`Sense::hover()`).
    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) = self.allocate(ui, Sense::hover());
        self.paint(ui, rect, self.value);
        response
    }

    /// Click, drag, and mouse wheel set `*value` in whole segments.
    pub fn show_interactive(self, ui: &mut Ui, value: &mut f32) -> Response {
        let (rect, mut response) = self.allocate(ui, Sense::click_and_drag());
        let n = self.segments;

        if (response.is_pointer_button_down_on() || response.clicked() || response.dragged())
            && let Some(pos) = response.interact_pointer_pos()
        {
            let new_value = value_at(rect, n, pos, self.vertical);
            if new_value != *value {
                *value = new_value;
                response.mark_changed();
            }
        }

        if response.hovered() && n > 0 {
            for event in ui.input(|i| i.events.clone()) {
                if let egui::Event::MouseWheel { delta, .. } = event {
                    // Step by whole segments from the *current* lit count, not
                    // by a fixed `1/n`, so an unaligned starting value still
                    // lands on a segment boundary.
                    let lit = lit_count(n, *value);
                    let new_lit = if delta.y > 0.0 {
                        Some((lit + 1).min(n))
                    } else if delta.y < 0.0 {
                        Some(lit.saturating_sub(1))
                    } else {
                        None
                    };
                    if let Some(new_lit) = new_lit {
                        let new_value = new_lit as f32 / n as f32;
                        if new_value != *value {
                            *value = new_value;
                            response.mark_changed();
                        }
                    }
                }
            }
        }

        self.paint(ui, rect, *value);
        response
    }

    fn allocate(&self, ui: &mut Ui, sense: Sense) -> (Rect, Response) {
        let size = self.desired_size.unwrap_or_else(|| self.default_size(ui));
        ui.allocate_exact_size(size, sense)
    }

    fn default_size(&self, ui: &Ui) -> Vec2 {
        let cell_w = cell_width(ui);
        if self.vertical {
            vec2(cell_w, self.segments as f32 * dots(ui.ctx(), 4.0))
        } else {
            vec2(
                self.segments as f32 * cell_w,
                ui.text_style_height(&TextStyle::Body),
            )
        }
    }

    fn paint(&self, ui: &Ui, rect: Rect, value: f32) {
        let ctx = ui.ctx();
        let palette = palette(ctx);
        let gap = dot(ctx);
        let lit = lit_count(self.segments, value);
        let painter = ui.painter();
        for (i, r) in segment_rects(rect, self.segments, gap, self.vertical)
            .into_iter()
            .enumerate()
        {
            let color = if i < lit {
                match self.warn {
                    Some((from, warn_color)) if i >= from => warn_color,
                    _ => self.fill.unwrap_or(palette.frame),
                }
            } else {
                palette.well
            };
            painter.rect_filled(snap_rect(ctx, r), 0, color);
        }
    }
}

/// Number of lit segments for `value` (clamped to `0..=1`) out of `segments`.
pub(crate) fn lit_count(segments: usize, value: f32) -> usize {
    (value.clamp(0.0, 1.0) * segments as f32).round() as usize
}

/// Rects for `n` equal segments spanning `rect` along its main axis, with a
/// `gap` between neighbours. Vertical: index 0 is the bottom segment.
pub(crate) fn segment_rects(rect: Rect, n: usize, gap: f32, vertical: bool) -> Vec<Rect> {
    if n == 0 {
        return Vec::new();
    }
    let len = if vertical {
        rect.height()
    } else {
        rect.width()
    };
    let step = (len + gap) / n as f32;
    let start = |i: usize| i as f32 * step;

    (0..n)
        .map(|i| {
            let s = start(i);
            let e = start(i + 1) - gap;
            if vertical {
                Rect::from_min_max(
                    pos2(rect.min.x, rect.max.y - e),
                    pos2(rect.max.x, rect.max.y - s),
                )
            } else {
                Rect::from_min_max(
                    pos2(rect.min.x + s, rect.min.y),
                    pos2(rect.min.x + e, rect.max.y),
                )
            }
        })
        .collect()
}

/// Value (`0..=1`, in whole `1/n` steps) for a pointer at `pos` inside `rect`,
/// rounding up to the segment the pointer is over.
pub(crate) fn value_at(rect: Rect, n: usize, pos: Pos2, vertical: bool) -> f32 {
    if n == 0 {
        return 0.0;
    }
    let frac = if vertical {
        (rect.max.y - pos.y) / rect.height()
    } else {
        (pos.x - rect.min.x) / rect.width()
    };
    let frac = frac.clamp(0.0, 1.0);
    (frac * n as f32).ceil() / n as f32
}

#[cfg(test)]
mod tests {
    use egui::{RawInput, pos2, vec2};

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    #[test]
    fn lit_count_rounds_and_clamps() {
        assert_eq!(lit_count(10, 0.04), 0);
        assert_eq!(lit_count(10, 0.05), 1);
        assert_eq!(lit_count(10, 1.5), 10);
        assert_eq!(lit_count(10, -1.0), 0);
    }

    #[test]
    fn segment_rects_count_and_gaps() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 10.0));
        let gap = 1.0;
        let rects = segment_rects(rect, 5, gap, false);
        assert_eq!(rects.len(), 5);
        for r in &rects {
            assert!(rect.contains_rect(*r));
        }
        for pair in rects.windows(2) {
            let sep = pair[1].min.x - pair[0].max.x;
            assert!(sep >= gap - 1e-4, "gap {sep} < {gap}");
        }
    }

    #[test]
    fn segment_rects_vertical_index0_bottom() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(10.0, 100.0));
        let rects = segment_rects(rect, 4, 1.0, true);
        // Index 0 sits lowest on screen (largest y), the last index highest.
        assert!(rects[0].max.y > rects[3].max.y);
        for r in &rects {
            assert!(rect.contains_rect(*r));
        }
    }

    #[test]
    fn value_at_ends() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 10.0));
        let left = value_at(rect, 10, pos2(0.0, 5.0), false);
        assert!(left == 0.0 || left == 0.1);
        let right = value_at(rect, 10, pos2(100.0, 5.0), false);
        assert_eq!(right, 1.0);
    }

    #[test]
    fn show_without_input_is_display_only() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            SegmentBar::new(8, 0.5).show(ui);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn show_interactive_without_input_keeps_value() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut value = 0.3f32;
        let mut changed = false;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let response = SegmentBar::new(8, 0.0).show_interactive(ui, &mut value);
            changed = response.changed();
        });
        output.textures_delta.clear();

        assert_eq!(value, 0.3);
        assert!(!changed);
    }
}
