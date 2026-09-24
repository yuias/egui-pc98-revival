//! 1-dot dither fill: a screen-aligned checkerboard drawn with a tiny
//! repeating texture.

use egui::{Color32, ColorImage, Id, Painter, Rect, TextureHandle, TextureOptions};

use crate::style::{dot, palette};

fn dither_texture_id() -> Id {
    Id::new("egui_pc98_revival::dither")
}

/// Fills `rect` with a screen-aligned 1-dot checkerboard of `fg`, over `bg` if given.
pub fn paint_dither(painter: &Painter, rect: Rect, fg: Color32, bg: Option<Color32>) {
    if let Some(bg) = bg {
        painter.rect_filled(rect, 0.0, bg);
    }

    let ctx = painter.ctx();
    // `load_texture` takes the context's own lock internally, so it cannot
    // run inside a `data_mut` closure (already holding that lock) without
    // deadlocking. Look the handle up first, and only load + store it
    // outside any lock when it is missing.
    let handle = ctx
        .data(|d| d.get_temp::<TextureHandle>(dither_texture_id()))
        .unwrap_or_else(|| {
            let image = ColorImage::new(
                [2, 2],
                vec![
                    Color32::WHITE,
                    Color32::TRANSPARENT,
                    Color32::TRANSPARENT,
                    Color32::WHITE,
                ],
            );
            let handle = ctx.load_texture("pc98_dither", image, TextureOptions::NEAREST_REPEAT);
            ctx.data_mut(|d| d.insert_temp(dither_texture_id(), handle.clone()));
            handle
        });

    painter.image(handle.id(), rect, dither_uv(rect, dot(ctx)), fg);
}

/// Dims everything under `rect` with a 1-dot checker of the palette's `ground`.
pub fn paint_scrim(painter: &Painter, rect: Rect) {
    paint_dither(painter, rect, palette(painter.ctx()).ground, None);
}

/// UV rect so one texel = one dot and texel (0,0) sits at the screen origin.
fn dither_uv(rect: Rect, dot: f32) -> Rect {
    Rect::from_min_max(
        (rect.min.to_vec2() / (2.0 * dot)).to_pos2(),
        (rect.max.to_vec2() / (2.0 * dot)).to_pos2(),
    )
}

#[cfg(test)]
mod tests {
    use egui::{RawInput, pos2, vec2};

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    #[test]
    fn dither_uv_unit_dot() {
        let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(8.0, 4.0));
        let uv = dither_uv(rect, 1.0);
        assert_eq!(uv.min, pos2(0.0, 0.0));
        assert_eq!(uv.max, pos2(4.0, 2.0));
    }

    #[test]
    fn dither_uv_scaled_dot() {
        let rect = Rect::from_min_max(pos2(4.0, 4.0), pos2(8.0, 8.0));
        let uv = dither_uv(rect, 2.0);
        assert_eq!(uv.min, pos2(1.0, 1.0));
        assert_eq!(uv.max, pos2(2.0, 2.0));
    }

    #[test]
    fn paint_dither_twice_reuses_one_texture() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let painter = ui.painter();
            let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(16.0, 16.0));
            paint_dither(painter, rect, Color32::RED, None);
            let first_id = ctx.data(|d| {
                d.get_temp::<TextureHandle>(dither_texture_id())
                    .unwrap()
                    .id()
            });
            paint_dither(painter, rect, Color32::BLUE, Some(Color32::BLACK));
            let second_id = ctx.data(|d| {
                d.get_temp::<TextureHandle>(dither_texture_id())
                    .unwrap()
                    .id()
            });
            assert_eq!(first_id, second_id);
        });
        // Loading the texture and laying out glyphs both produce texture
        // deltas; consume them so the test does not leak the delta.
        output.textures_delta.clear();
    }

    #[test]
    fn paint_scrim_smoke_test() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let painter = ui.painter();
            let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(16.0, 16.0));
            paint_scrim(painter, rect);
        });
        output.textures_delta.clear();
    }
}
