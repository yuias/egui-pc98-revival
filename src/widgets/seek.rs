//! Horizontal seek bar: dithered track, filled position, 2-dot handle, ticks.

use egui::{Painter, Rect, Response, Sense, TextStyle, Ui, pos2, vec2};

use crate::Palette;
use crate::style::{dots, palette, snap_rect};
use crate::widgets::paint_dither;

/// Horizontal seek bar over the available width: dithered track, filled part,
/// 2-dot handle, tick marks below. Click or drag sets `*value` in `0..=1`.
pub fn seek_bar(ui: &mut Ui, value: &mut f32, ticks: &[f32]) -> Response {
    let ctx = ui.ctx().clone();
    let pal = palette(&ctx);
    let size = vec2(ui.available_width(), ui.text_style_height(&TextStyle::Body));
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let track = track_rect(&ctx, rect);

    if (response.is_pointer_button_down_on() || response.clicked() || response.dragged())
        && let Some(pos) = response.interact_pointer_pos()
    {
        let new_value = seek_value_at(track, pos.x);
        if new_value != *value {
            *value = new_value;
            response.mark_changed();
        }
    }

    let painter = ui.painter();
    paint_dither(painter, snap_rect(&ctx, track), pal.dim, Some(pal.well));

    // A NaN value (e.g. before the caller has set an initial position) is
    // drawn as the start of the track rather than propagated into geometry.
    let paint_value = if value.is_nan() { 0.0 } else { *value };
    paint_fill_and_handle(painter, &ctx, pal, track, paint_value, &response);
    paint_ticks(painter, &ctx, pal, track, ticks);

    response
}

fn track_rect(ctx: &egui::Context, r: Rect) -> Rect {
    Rect::from_min_max(
        pos2(r.left() + dots(ctx, 1.0), r.top() + dots(ctx, 3.0)),
        pos2(r.right() - dots(ctx, 1.0), r.top() + dots(ctx, 9.0)),
    )
}

fn paint_fill_and_handle(
    painter: &Painter,
    ctx: &egui::Context,
    pal: Palette,
    track: Rect,
    value: f32,
    response: &Response,
) {
    let x = seek_x(track, value);

    let filled = Rect::from_min_max(track.min, pos2(x, track.max.y));
    painter.rect_filled(snap_rect(ctx, filled), 0, pal.frame);

    let handle_w = dots(ctx, 2.0);
    let inset = dots(ctx, 2.0);
    let handle = Rect::from_min_max(
        pos2(x - handle_w / 2.0, track.top() - inset),
        pos2(x + handle_w / 2.0, track.bottom() + inset),
    );
    let color = if response.hovered() || response.dragged() {
        pal.accent
    } else {
        pal.text
    };
    painter.rect_filled(snap_rect(ctx, handle), 0, color);
}

fn paint_ticks(painter: &Painter, ctx: &egui::Context, pal: Palette, track: Rect, ticks: &[f32]) {
    let tick_w = dots(ctx, 1.0);
    for &t in ticks {
        if !(0.0..=1.0).contains(&t) {
            continue;
        }
        let x = seek_x(track, t);
        let tick = Rect::from_min_max(
            pos2(x - tick_w / 2.0, track.bottom() + dots(ctx, 2.0)),
            pos2(x + tick_w / 2.0, track.bottom() + dots(ctx, 5.0)),
        );
        painter.rect_filled(snap_rect(ctx, tick), 0, pal.dim);
    }
}

/// `((x - left) / width).clamp(0, 1)`; a non-positive track width maps to 0.
pub(crate) fn seek_value_at(track: Rect, x: f32) -> f32 {
    let width = track.width();
    if width <= 0.0 {
        return 0.0;
    }
    ((x - track.left()) / width).clamp(0.0, 1.0)
}

/// `left + value.clamp(0, 1) * width`.
pub(crate) fn seek_x(track: Rect, value: f32) -> f32 {
    track.left() + value.clamp(0.0, 1.0) * track.width()
}

#[cfg(test)]
mod tests {
    use egui::{Event, Modifiers, PointerButton, RawInput};

    use crate::style::apply_with;

    use super::*;

    fn track(left: f32, right: f32) -> Rect {
        Rect::from_min_max(pos2(left, 0.0), pos2(right, 6.0))
    }

    #[test]
    fn seek_value_at_clamps() {
        let track = track(0.0, 100.0);
        assert_eq!(seek_value_at(track, -10.0), 0.0);
        assert_eq!(seek_value_at(track, 110.0), 1.0);
        assert_eq!(seek_value_at(track, 50.0), 0.5);
    }

    #[test]
    fn seek_value_at_zero_width_is_zero() {
        let track = track(10.0, 10.0);
        assert_eq!(seek_value_at(track, 10.0), 0.0);
        assert_eq!(seek_value_at(track, 100.0), 0.0);
    }

    #[test]
    fn seek_x_roundtrip() {
        let track = track(20.0, 220.0);
        for v in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let x = seek_x(track, v);
            let back = seek_value_at(track, x);
            assert!((back - v).abs() < 1e-4, "{back} != {v}");
        }
    }

    #[test]
    fn seek_bar_no_input_unchanged() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut value = 0.3f32;
        let mut changed = false;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let response = seek_bar(ui, &mut value, &[0.25, 0.5, 0.75]);
            changed = response.changed();
        });
        output.textures_delta.clear();

        assert_eq!(value, 0.3);
        assert!(!changed);
    }

    #[test]
    fn seek_bar_click_sets_value() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut value = 0.0f32;

        // Warm-up frames: hit-testing uses the rect registered two passes ago.
        let mut rect = Rect::NOTHING;
        for _ in 0..2 {
            let mut output = ctx.run_ui(RawInput::default(), |ui| {
                rect = seek_bar(ui, &mut value, &[]).rect;
            });
            output.textures_delta.clear();
        }

        let pos = pos2(rect.left() + rect.width() * 0.75, rect.center().y);

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
                let response = seek_bar(ui, &mut value, &[]);
                changed = response.changed();
            },
        );
        output.textures_delta.clear();

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
                seek_bar(ui, &mut value, &[]);
            },
        );
        output.textures_delta.clear();

        assert!((value - 0.75).abs() < 0.02, "value = {value}");
        assert!(changed);
    }
}
