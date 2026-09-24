//! Glyph-free icons drawn from axis-aligned dot rects, and a framed icon
//! button. Icons avoid the font because the bundled font draws box-drawing
//! and arrow glyphs full-width or not at all (see `text::char_cells`).

use egui::{Color32, Painter, Rect, Response, Sense, TextStyle, Ui, vec2};

use crate::style::{dot, dots, palette, snap_rect};
use crate::widgets::paint_frame;

/// Glyph-free icon drawn from axis-aligned dot rects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DotIcon {
    Play,
    Pause,
    Stop,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Check,
}

/// Side of the square grid `icon_cells` places cells on.
pub(crate) const ICON_GRID: u8 = 7;

/// Cells for `icon` on a `ICON_GRID`x`ICON_GRID` grid, as `(x, y, w, h)`, y down.
pub(crate) fn icon_cells(icon: DotIcon) -> Vec<[u8; 4]> {
    match icon {
        DotIcon::Play => vec![[2, 0, 1, 7], [3, 1, 1, 5], [4, 2, 1, 3], [5, 3, 1, 1]],
        DotIcon::Pause => vec![[1, 0, 2, 7], [4, 0, 2, 7]],
        DotIcon::Stop => vec![[1, 1, 5, 5]],
        DotIcon::ArrowRight => arrow_right_cells(),
        // Arrow variants are derived from `ArrowRight` so they cannot drift apart.
        DotIcon::ArrowLeft => mirror_cells(&arrow_right_cells()),
        DotIcon::ArrowUp => transpose_cells(&mirror_cells(&arrow_right_cells())),
        DotIcon::ArrowDown => transpose_cells(&arrow_right_cells()),
        DotIcon::Check => (0u8..7)
            .zip([2u8, 3, 4, 3, 2, 1, 0])
            .map(|(x, y)| [x, y, 1, 2])
            .collect(),
    }
}

fn arrow_right_cells() -> Vec<[u8; 4]> {
    vec![
        [0, 2, 3, 3],
        [3, 0, 1, 7],
        [4, 1, 1, 5],
        [5, 2, 1, 3],
        [6, 3, 1, 1],
    ]
}

/// Mirrors cells horizontally within the grid: `x' = GRID - x - w`.
fn mirror_cells(cells: &[[u8; 4]]) -> Vec<[u8; 4]> {
    cells
        .iter()
        .map(|&[x, y, w, h]| [ICON_GRID - x - w, y, w, h])
        .collect()
}

/// Transposes cells (swaps x/y and w/h), turning a horizontal icon vertical.
fn transpose_cells(cells: &[[u8; 4]]) -> Vec<[u8; 4]> {
    cells.iter().map(|&[x, y, w, h]| [y, x, h, w]).collect()
}

/// Paints `icon` centred in `rect`, scaled by a whole number of dots.
pub fn paint_dot_icon(painter: &Painter, rect: Rect, icon: DotIcon, color: Color32) {
    let ctx = painter.ctx();
    let d = dot(ctx);
    // At least 1 dot per grid cell; a rect smaller than the 7x7 grid still
    // draws at that minimum and is clipped by the painter.
    let unit = d
        * (rect.width().min(rect.height()) / (ICON_GRID as f32 * d))
            .floor()
            .max(1.0);
    let origin = rect.center() - vec2(3.5, 3.5) * unit;
    for [x, y, w, h] in icon_cells(icon) {
        let cell = Rect::from_min_size(
            origin + vec2(x as f32 * unit, y as f32 * unit),
            vec2(w as f32 * unit, h as f32 * unit),
        );
        painter.rect_filled(snap_rect(ctx, cell), 0, color);
    }
}

/// Square button with a 1-dot frame and a dot icon.
pub fn icon_button(ui: &mut Ui, icon: DotIcon) -> Response {
    let ctx = ui.ctx().clone();
    let palette = palette(&ctx);
    let side = ui.text_style_height(&TextStyle::Body);
    let (rect, response) = ui.allocate_exact_size(vec2(side, side), Sense::click());

    let enabled = ui.is_enabled();
    let active = response.hovered() || response.has_focus() || response.is_pointer_button_down_on();
    let (fill, frame_color, icon_color) = if !enabled {
        (None, palette.dim, palette.dim)
    } else if active {
        (Some(palette.frame), palette.frame, palette.ground)
    } else {
        (None, palette.frame, palette.text)
    };

    let rect = snap_rect(&ctx, rect);
    let painter = ui.painter();
    if let Some(fill) = fill {
        painter.rect_filled(rect, 0, fill);
    }
    paint_frame(painter, rect, frame_color);

    let icon_rect = rect.shrink(dots(&ctx, 3.0));
    paint_dot_icon(painter, icon_rect, icon, icon_color);

    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, format!("{icon:?}"))
    });

    response
}

#[cfg(test)]
mod tests {
    use egui::{RawInput, pos2, vec2};

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    #[test]
    fn icon_cells_inside_grid() {
        let icons = [
            DotIcon::Play,
            DotIcon::Pause,
            DotIcon::Stop,
            DotIcon::ArrowUp,
            DotIcon::ArrowDown,
            DotIcon::ArrowLeft,
            DotIcon::ArrowRight,
            DotIcon::Check,
        ];
        for icon in icons {
            for [x, y, w, h] in icon_cells(icon) {
                assert!(w >= 1 && h >= 1, "{icon:?}: cell [{x},{y},{w},{h}]");
                assert!(
                    x + w <= ICON_GRID && y + h <= ICON_GRID,
                    "{icon:?}: cell [{x},{y},{w},{h}] out of bounds"
                );
            }
        }
    }

    #[test]
    fn arrow_left_mirrors_right() {
        let right = icon_cells(DotIcon::ArrowRight);
        let left = icon_cells(DotIcon::ArrowLeft);
        assert_eq!(right.len(), left.len());
        for (r, l) in right.iter().zip(left.iter()) {
            assert_eq!(l, &[ICON_GRID - r[0] - r[2], r[1], r[2], r[3]]);
        }
    }

    #[test]
    fn arrow_up_transposes_left() {
        let left = icon_cells(DotIcon::ArrowLeft);
        let up = icon_cells(DotIcon::ArrowUp);
        assert_eq!(left.len(), up.len());
        for (l, u) in left.iter().zip(up.iter()) {
            assert_eq!(u, &[l[1], l[0], l[3], l[2]]);
        }
    }

    #[test]
    fn arrow_down_transposes_right() {
        let right = icon_cells(DotIcon::ArrowRight);
        let down = icon_cells(DotIcon::ArrowDown);
        assert_eq!(right.len(), down.len());
        for (r, d) in right.iter().zip(down.iter()) {
            assert_eq!(d, &[r[1], r[0], r[3], r[2]]);
        }
    }

    #[test]
    fn play_differs_from_arrow_right() {
        assert_ne!(icon_cells(DotIcon::Play), icon_cells(DotIcon::ArrowRight));
    }

    #[test]
    fn icon_button_smoke() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut response = None;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            response = Some(icon_button(ui, DotIcon::Play));
        });
        output.textures_delta.clear();

        let response = response.unwrap();
        assert!(!response.clicked());
        assert!(response.rect.width() > 0.0);
        assert!((response.rect.width() - response.rect.height()).abs() < 1e-3);
    }

    #[test]
    fn paint_dot_icon_small_rect_does_not_panic() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let painter = ui.painter();
            let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(3.0, 3.0));
            paint_dot_icon(painter, rect, DotIcon::Stop, Color32::WHITE);
        });
        output.textures_delta.clear();
    }
}
