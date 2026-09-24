//! One-cell spinner cycling `| / - \` shapes. Drawn from axis-aligned dot
//! rects rather than font glyphs, because the bundled JIS font draws `\` as
//! `¥`.

use std::time::Duration;

use egui::{Color32, Painter, Rect, Response, Sense, TextStyle, Ui, vec2};

use crate::style::{dot, snap_rect};
use crate::text;

const FRAMES: [char; 4] = ['|', '/', '-', '\\'];
const FRAME_SECS: f64 = 0.125;

/// Side of the square grid each spinner shape is drawn on.
const SPINNER_GRID: u8 = 5;

/// Frame to show at `time` and seconds until the next frame starts.
fn spinner_frame(time: f64) -> (char, f64) {
    let time = if time.is_finite() && time >= 0.0 {
        time
    } else {
        0.0
    };
    let index = (time / FRAME_SECS) as usize % FRAMES.len();
    let remaining = (FRAME_SECS - time % FRAME_SECS).max(1e-3);
    (FRAMES[index], remaining)
}

/// Cells for `frame` on a `SPINNER_GRID`x`SPINNER_GRID` grid, as `(x, y, w, h)`, y down.
/// Diagonals are staircases of 1-dot cells rather than diagonal strokes.
fn spinner_cells(frame: char) -> Vec<[u8; 4]> {
    match frame {
        '|' => vec![[2, 0, 1, SPINNER_GRID]],
        '-' => vec![[0, 2, SPINNER_GRID, 1]],
        '/' => (0..SPINNER_GRID)
            .map(|x| [x, SPINNER_GRID - 1 - x, 1, 1])
            .collect(),
        '\\' => (0..SPINNER_GRID).map(|x| [x, x, 1, 1]).collect(),
        _ => Vec::new(),
    }
}

/// Paints one spinner `frame` centred in `rect`, scaled to fill it.
fn paint_spinner_frame(painter: &Painter, rect: Rect, frame: char, color: Color32) {
    let ctx = painter.ctx();
    let d = dot(ctx);
    let grid = SPINNER_GRID as f32;
    // Independent x/y scale, unlike the square icon grid: the allocated cell
    // is a text cell (narrower than it is tall), not a square button.
    let unit_x = d * (rect.width() / (grid * d)).floor().max(1.0);
    let unit_y = d * (rect.height() / (grid * d)).floor().max(1.0);
    let origin = rect.center() - vec2(grid / 2.0 * unit_x, grid / 2.0 * unit_y);
    for [x, y, w, h] in spinner_cells(frame) {
        let cell = Rect::from_min_size(
            origin + vec2(x as f32 * unit_x, y as f32 * unit_y),
            vec2(w as f32 * unit_x, h as f32 * unit_y),
        );
        painter.rect_filled(snap_rect(ctx, cell), 0, color);
    }
}

/// A one-cell spinner cycling `| / - \` shapes; schedules its own repaints.
pub fn text_spinner(ui: &mut Ui) -> Response {
    let time = ui.input(|i| i.time);
    let (frame, remaining) = spinner_frame(time);
    let row_height = ui.text_style_height(&TextStyle::Body);
    let size = vec2(text::cell_width(ui), row_height);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());

    let color = ui.visuals().text_color();
    paint_spinner_frame(ui.painter(), rect, frame, color);
    ui.ctx()
        .request_repaint_after(Duration::from_secs_f64(remaining));

    response
}

#[cfg(test)]
mod tests {
    use egui::RawInput;

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    #[test]
    fn spinner_frame_cycles_through_all_frames() {
        let times = [0.01, 0.135, 0.26, 0.385, 0.51];
        let expected = ['|', '/', '-', '\\', '|'];
        for (time, want) in times.iter().zip(expected) {
            let (frame, _) = spinner_frame(*time);
            assert_eq!(frame, want, "time {time}");
        }
    }

    #[test]
    fn spinner_frame_remaining_is_positive_and_bounded() {
        let mut time = 0.0;
        while time < 2.0 {
            let (_, remaining) = spinner_frame(time);
            assert!(
                remaining > 0.0 && remaining <= FRAME_SECS,
                "time {time}: remaining {remaining}"
            );
            time += 0.013;
        }
    }

    #[test]
    fn spinner_frame_handles_bad_time() {
        assert_eq!(spinner_frame(f64::NAN).0, '|');
        assert_eq!(spinner_frame(-1.0).0, '|');
    }

    #[test]
    fn text_spinner_allocates_one_cell() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut response = None;
        let mut cell_width = 0.0;
        let mut output = ctx.run_ui(
            RawInput {
                time: Some(0.3),
                ..Default::default()
            },
            |ui| {
                cell_width = text::cell_width(ui);
                response = Some(text_spinner(ui));
            },
        );
        output.textures_delta.clear();

        let response = response.unwrap();
        assert!((response.rect.width() - cell_width).abs() < 0.01);
    }
}
