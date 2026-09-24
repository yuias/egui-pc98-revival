//! Fixed-width scrolling label: pauses, scrolls one cell at a time, loops.

use std::time::Duration;

use egui::{Align2, Color32, Response, Sense, TextStyle, Ui, vec2};

use crate::text;

const PAUSE_SECS: f64 = 1.0;
const STEP_SECS: f64 = 0.15;

/// Scroll step at `time` for a ring of `ring_cells` cells, and seconds until it changes.
fn marquee_phase(time: f64, ring_cells: usize) -> (usize, f64) {
    let cycle = PAUSE_SECS + ring_cells as f64 * STEP_SECS;
    let t = if time.is_finite() && time >= 0.0 {
        time
    } else {
        0.0
    };
    let phase = t % cycle;
    if phase < PAUSE_SECS {
        return (0, PAUSE_SECS - phase);
    }
    let elapsed = phase - PAUSE_SECS;
    let step = ((elapsed / STEP_SECS) as usize).min(ring_cells.saturating_sub(1));
    let remaining = (STEP_SECS - elapsed % STEP_SECS).max(1e-3);
    (step, remaining)
}

/// A `max_cells`-wide label; text wider than that pauses, scrolls left one
/// cell at a time, and loops. Schedules its own repaints while scrolling.
pub fn marquee(ui: &mut Ui, text: &str, max_cells: usize, color: Color32) -> Response {
    let row_height = ui.text_style_height(&TextStyle::Body);
    let size = vec2(max_cells as f32 * text::cell_width(ui), row_height);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());

    let font = TextStyle::Body.resolve(ui.style());
    if text::cells(text) <= max_cells {
        ui.painter_at(rect)
            .text(rect.left_top(), Align2::LEFT_TOP, text, font, color);
        return response;
    }

    let ring_cells = text::cells(text) + 3;
    let (step, remaining) = marquee_phase(ui.input(|i| i.time), ring_cells);
    let slice = text::marquee_slice(text, max_cells, step);
    ui.painter_at(rect)
        .text(rect.left_top(), Align2::LEFT_TOP, slice, font, color);
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
    fn marquee_phase_pauses_steps_and_wraps() {
        let ring_cells = 9; // cycle = 1.0 + 9 * 0.15 = 2.35
        let cycle = PAUSE_SECS + ring_cells as f64 * STEP_SECS;

        let (step, remaining) = marquee_phase(0.0, ring_cells);
        assert_eq!(step, 0);
        assert!((remaining - 1.0).abs() < 1e-9);

        assert_eq!(marquee_phase(1.0 + 0.5 * STEP_SECS, ring_cells).0, 0);
        assert_eq!(marquee_phase(1.0 + 2.5 * STEP_SECS, ring_cells).0, 2);
        assert_eq!(marquee_phase(1.0 + 8.5 * STEP_SECS, ring_cells).0, 8);
        assert_eq!(marquee_phase(cycle + 0.1, ring_cells).0, 0);

        let (step, remaining) = marquee_phase(f64::NAN, ring_cells);
        assert_eq!(step, 0);
        assert!((remaining - PAUSE_SECS).abs() < 1e-9);
        let (step, remaining) = marquee_phase(-1.0, ring_cells);
        assert_eq!(step, 0);
        assert!((remaining - PAUSE_SECS).abs() < 1e-9);
    }

    #[test]
    fn marquee_phase_remaining_and_step_are_bounded() {
        let ring_cells = 9;
        let mut time = 0.0;
        while time < 5.0 {
            let (step, remaining) = marquee_phase(time, ring_cells);
            assert!(
                remaining > 0.0 && remaining <= PAUSE_SECS,
                "time {time}: remaining {remaining}"
            );
            assert!(step < ring_cells, "time {time}: step {step}");
            time += 0.037;
        }
    }

    #[test]
    fn marquee_short_text_is_static() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        for t in [0.0, 1.3, 2.0] {
            let mut response = None;
            let mut cell_width = 0.0;
            let mut output = ctx.run_ui(
                RawInput {
                    time: Some(t),
                    ..Default::default()
                },
                |ui| {
                    cell_width = text::cell_width(ui);
                    response = Some(marquee(ui, "short", 8, Color32::WHITE));
                },
            );
            output.textures_delta.clear();

            let response = response.unwrap();
            assert!(
                (response.rect.width() - 8.0 * cell_width).abs() < 0.01,
                "t {t}"
            );
        }
    }

    #[test]
    fn marquee_long_text_keeps_constant_width() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let text = "abcdefghijklmnopqrst"; // 20 cells
        for t in [0.0, 1.3, 2.0] {
            let mut response = None;
            let mut cell_width = 0.0;
            let mut output = ctx.run_ui(
                RawInput {
                    time: Some(t),
                    ..Default::default()
                },
                |ui| {
                    cell_width = text::cell_width(ui);
                    response = Some(marquee(ui, text, 8, Color32::WHITE));
                },
            );
            output.textures_delta.clear();

            let response = response.unwrap();
            assert!(
                (response.rect.width() - 8.0 * cell_width).abs() < 0.01,
                "t {t}"
            );
        }
    }
}
