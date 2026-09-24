//! Full-width header bar.

use egui::{Color32, Rect, Response, Sense, TextStyle, Ui, pos2, vec2};

use crate::layout::fit_by_priority;
use crate::style::{dot, dots, palette, snap_rect};
use crate::text::{cell_width, cells};

/// One right-side item of a [`HeaderBar`].
#[derive(Clone, Copy, Debug)]
struct HeaderItem<'a> {
    text: &'a str,
    color: Option<Color32>,
    drop_priority: Option<u8>,
}

/// Full-width title/status bar: optional accent badge, left text, and
/// right-aligned items separated by 1-dot rules; items drop by priority when narrow.
#[derive(Clone, Debug, Default)]
pub struct HeaderBar<'a> {
    badge: Option<&'a str>,
    left: Option<&'a str>,
    items: Vec<HeaderItem<'a>>,
}

impl<'a> HeaderBar<'a> {
    /// Empty bar: no badge, no left text, no items.
    pub fn new() -> Self {
        Self::default()
    }

    /// Short label in an `accent` block at the left edge.
    pub fn badge(mut self, text: &'a str) -> Self {
        self.badge = Some(text);
        self
    }

    /// Sets the left-aligned text.
    pub fn left(mut self, text: &'a str) -> Self {
        self.left = Some(text);
        self
    }

    /// Appends a right-side item. `color` defaults to `bar_fg`; `drop_priority`
    /// `None` = never dropped, higher values drop first (see `fit_by_priority`).
    pub fn item(
        mut self,
        text: &'a str,
        color: Option<Color32>,
        drop_priority: Option<u8>,
    ) -> Self {
        self.items.push(HeaderItem {
            text,
            color,
            drop_priority,
        });
        self
    }

    /// Draws the bar and returns its (non-interactive) response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let ctx = ui.ctx();
        let palette = palette(ctx);
        let font_id = TextStyle::Body.resolve(ui.style());
        let cell_w = cell_width(ui);
        let d = dot(ctx);

        let outer = ui.available_rect_before_wrap();
        let height = ui.text_style_height(&TextStyle::Body) + dots(ctx, 4.0);
        let bar_rect = Rect::from_min_size(outer.min, vec2(ui.available_width(), height));

        let painter = ui.painter();
        painter.rect_filled(bar_rect, 0, palette.bar_bg);

        // Right edge of the left text (or of the badge, or `bar.left` if
        // neither is set); only used to size the space left for items.
        let mut left_right = bar_rect.left();
        let mut badge_right = None;

        if let Some(badge) = self.badge {
            let badge_w = (cells(badge) + 2) as f32 * cell_w;
            let badge_rect = Rect::from_min_size(bar_rect.min, vec2(badge_w, height));
            painter.rect_filled(badge_rect, 0, palette.accent);
            let galley =
                painter.layout_no_wrap(badge.to_owned(), font_id.clone(), palette.hover_fg);
            let pos = badge_rect.center() - galley.size() / 2.0;
            painter.galley(pos, galley, palette.hover_fg);
            badge_right = Some(badge_rect.right());
            left_right = badge_rect.right();
        }

        if let Some(left) = self.left {
            let x = match badge_right {
                Some(right) => right + cell_w,
                None => bar_rect.left() + dots(ctx, 8.0),
            };
            let galley = painter.layout_no_wrap(left.to_owned(), font_id.clone(), palette.bar_fg);
            let y = bar_rect.center().y - galley.size().y / 2.0;
            left_right = x + galley.size().x;
            painter.galley(pos2(x, y), galley, palette.bar_fg);
        }

        let item_galleys: Vec<_> = self
            .items
            .iter()
            .map(|item| {
                let color = item.color.unwrap_or(palette.bar_fg);
                painter.layout_no_wrap(item.text.to_owned(), font_id.clone(), color)
            })
            .collect();
        let widths: Vec<f32> = item_galleys.iter().map(|g| g.size().x).collect();
        let drop_priority: Vec<Option<u8>> = self.items.iter().map(|i| i.drop_priority).collect();

        let gap = 2.0 * cell_w + d;
        let right_edge = bar_rect.right() - dots(ctx, 8.0);
        let available = (right_edge - (left_right + 2.0 * cell_w)).max(0.0);

        let keep = fit_by_priority(&widths, &drop_priority, gap, available);
        let positions = item_positions(&widths, &keep, gap, right_edge);

        let mut prev_right: Option<f32> = None;
        for (i, galley) in item_galleys.into_iter().enumerate() {
            let Some(x) = positions[i] else {
                continue;
            };

            if let Some(prev_right) = prev_right {
                let rule_x = (prev_right + x) / 2.0;
                let rule_rect = Rect::from_min_max(
                    pos2(rule_x - d / 2.0, bar_rect.top() + dots(ctx, 2.0)),
                    pos2(rule_x + d / 2.0, bar_rect.bottom() - dots(ctx, 2.0)),
                );
                painter.rect_filled(snap_rect(ctx, rule_rect), 0, palette.bar_fg);
            }

            let color = self.items[i].color.unwrap_or(palette.bar_fg);
            let y = bar_rect.center().y - galley.size().y / 2.0;
            prev_right = Some(x + galley.size().x);
            painter.galley(pos2(x, y), galley, color);
        }

        ui.allocate_rect(bar_rect, Sense::hover())
    }
}

/// Left x of each kept item, right-aligned so the last kept item ends at
/// `right`; `None` for a dropped item. `gap` is only counted between two
/// kept neighbours. Pure, so the priority-drop layout is testable without a `Ui`.
pub(crate) fn item_positions(
    widths: &[f32],
    keep: &[bool],
    gap: f32,
    right: f32,
) -> Vec<Option<f32>> {
    let mut positions = vec![None; widths.len()];
    let mut cursor = right;
    let mut first = true;
    for i in (0..widths.len()).rev() {
        if !keep[i] {
            continue;
        }
        if !first {
            cursor -= gap;
        }
        cursor -= widths[i];
        positions[i] = Some(cursor);
        first = false;
    }
    positions
}

/// Shorthand for `HeaderBar::new().left(left)` plus one right item.
pub fn header_bar(ui: &mut Ui, left: &str, right: Option<&str>) -> Response {
    let mut bar = HeaderBar::new().left(left);
    if let Some(right) = right {
        bar = bar.item(right, None, None);
    }
    bar.show(ui)
}

#[cfg(test)]
mod tests {
    use egui::RawInput;

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    #[test]
    fn item_positions_right_aligned() {
        let widths = [10.0, 20.0, 15.0];
        let keep = [true, true, true];
        let positions = item_positions(&widths, &keep, 5.0, 100.0);
        let last = positions[2].unwrap();
        assert!((last + widths[2] - 100.0).abs() < 1e-6);
    }

    #[test]
    fn item_positions_skip_dropped() {
        let widths = [10.0, 20.0, 15.0];
        let keep = [true, false, true];
        let gap = 5.0;
        let right = 100.0;
        let positions = item_positions(&widths, &keep, gap, right);

        assert_eq!(positions[1], None);
        let x2 = positions[2].unwrap();
        assert!((x2 + widths[2] - right).abs() < 1e-6);
        // Gap is counted only once, between the two kept neighbours (0 and 2).
        let x0 = positions[0].unwrap();
        assert!((x0 + widths[0] + gap - x2).abs() < 1e-6);
    }

    #[test]
    fn header_bar_builder_smoke() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut builder_response = None;
        let mut plain_response = None;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let palette = palette(ui.ctx());
            builder_response = Some(
                HeaderBar::new()
                    .badge("98")
                    .left("Title")
                    .item("A:", None, Some(2))
                    .item("640KB", Some(palette.ok), Some(1))
                    .item("12:00", None, None)
                    .show(ui),
            );
            plain_response = Some(header_bar(ui, "Title", Some("12:00")));
        });
        output.textures_delta.clear();

        let builder_response = builder_response.unwrap();
        let plain_response = plain_response.unwrap();
        assert!((builder_response.rect.height() - plain_response.rect.height()).abs() < 1e-3);
    }

    #[test]
    fn header_bar_builder_narrow_drops_items() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            ui.allocate_ui(vec2(120.0, 40.0), |ui| {
                HeaderBar::new()
                    .badge("98")
                    .left("Title")
                    .item("A:", None, Some(2))
                    .item("640KB", None, Some(1))
                    .item("12:00", None, None)
                    .show(ui);
            });
        });
        output.textures_delta.clear();
    }
}
