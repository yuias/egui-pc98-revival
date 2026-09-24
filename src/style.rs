//! Dot-grid math, the pure `egui::Style` builder, and context integration
//! that keeps a PC-98 style installed as the display scale changes.

use egui::style::HandleShape;
use egui::{CornerRadius, FontFamily, FontId, Id, Rect, Shadow, Stroke, ThemePreference, pos2};

use crate::Palette;

/// Native cell height of the bundled font, in dots.
pub const FONT_DOTS: f32 = 16.0;

/// Integer device pixels per dot for a given `pixels_per_point` (never below 1).
pub fn dot_px(ppp: f32) -> f32 {
    ppp.round().max(1.0)
}

/// Size of one dot in logical points.
pub fn dot_for_ppp(ppp: f32) -> f32 {
    dot_px(ppp) / ppp
}

/// Font size in points so that the 16-dot font renders at exactly 16 dots.
pub fn font_size_for_ppp(ppp: f32) -> f32 {
    FONT_DOTS * dot_for_ppp(ppp)
}

/// Dot size for the context's current scale.
pub fn dot(ctx: &egui::Context) -> f32 {
    dot_for_ppp(ctx.pixels_per_point())
}

/// `n` dots in logical points at the context's current scale.
pub fn dots(ctx: &egui::Context, n: f32) -> f32 {
    n * dot(ctx)
}

/// Rounds every edge of `rect` to the nearest device pixel for `ppp`. Pure,
/// so it can be unit-tested without a `Context`.
pub fn snap_rect_for_ppp(rect: Rect, ppp: f32) -> Rect {
    let snap = |v: f32| (v * ppp).round() / ppp;
    Rect::from_min_max(
        pos2(snap(rect.min.x), snap(rect.min.y)),
        pos2(snap(rect.max.x), snap(rect.max.y)),
    )
}

/// `snap_rect_for_ppp` with the context's `pixels_per_point`.
pub fn snap_rect(ctx: &egui::Context, rect: Rect) -> Rect {
    snap_rect_for_ppp(rect, ctx.pixels_per_point())
}

/// Full PC-98 style for the given palette and scale. Pure; no `Context` needed.
pub fn pc98_style(palette: &Palette, ppp: f32) -> egui::Style {
    let mut style = egui::Style {
        visuals: egui::Visuals::dark(),
        ..Default::default()
    };

    let d = dot_for_ppp(ppp);
    let v = &mut style.visuals;

    v.window_corner_radius = CornerRadius::ZERO;
    v.menu_corner_radius = CornerRadius::ZERO;
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;
    v.window_fill = palette.ground;
    v.panel_fill = palette.ground;
    v.window_stroke = Stroke::new(d, palette.frame);
    v.extreme_bg_color = palette.well;
    v.text_edit_bg_color = Some(palette.well);
    v.code_bg_color = palette.well;
    v.faint_bg_color = palette.well;
    v.hyperlink_color = palette.frame;
    v.warn_fg_color = palette.warn;
    v.error_fg_color = palette.danger;
    v.selection.bg_fill = palette.selected_bg;
    v.selection.stroke = Stroke::new(d, palette.bar_fg);
    v.handle_shape = HandleShape::Rect { aspect_ratio: 0.5 };

    // Deliberate deviation from a naive port: `Style::widget_style` prefers
    // `override_text_color` over the per-state text color, which would draw
    // hovered buttons with light text on the yellow hover fill.
    v.override_text_color = None;

    let w = &mut v.widgets;
    for state in [
        &mut w.noninteractive,
        &mut w.inactive,
        &mut w.hovered,
        &mut w.active,
        &mut w.open,
    ] {
        state.corner_radius = CornerRadius::ZERO;
        state.expansion = 0.0;
    }

    w.noninteractive.bg_fill = palette.ground;
    w.noninteractive.weak_bg_fill = palette.ground;
    w.noninteractive.bg_stroke = Stroke::new(d, palette.frame);
    w.noninteractive.fg_stroke = Stroke::new(d, palette.text);

    // Also the scroll handle and slider rail, which sit on `extreme_bg_color`
    // (`well`) and would vanish if this were `well` too.
    w.inactive.bg_fill = palette.dim;
    w.inactive.weak_bg_fill = palette.ground;
    w.inactive.bg_stroke = Stroke::new(d, palette.frame);
    w.inactive.fg_stroke = Stroke::new(d, palette.text);

    w.hovered.bg_fill = palette.accent;
    w.hovered.weak_bg_fill = palette.accent;
    w.hovered.bg_stroke = Stroke::new(d, palette.accent);
    w.hovered.fg_stroke = Stroke::new(d, palette.hover_fg);

    w.active.bg_fill = palette.frame;
    w.active.weak_bg_fill = palette.frame;
    w.active.bg_stroke = Stroke::new(d, palette.frame);
    w.active.fg_stroke = Stroke::new(d, palette.hover_fg);

    // `open.weak_bg_fill` also fills the focused window's title bar, whose
    // text egui draws in the plain text color, so this pair must contrast
    // with `text` rather than with black.
    w.open.bg_fill = palette.bar_bg;
    w.open.weak_bg_fill = palette.bar_bg;
    w.open.bg_stroke = Stroke::new(d, palette.frame);
    w.open.fg_stroke = Stroke::new(d, palette.bar_fg);

    style.spacing.scroll = egui::style::ScrollStyle::solid();
    // The edge fade blends content into the background, i.e. off-palette colors.
    style.spacing.scroll.fade.strength = 0.0;

    let font_id = FontId::new(font_size_for_ppp(ppp), FontFamily::Monospace);
    for size in style.text_styles.values_mut() {
        *size = font_id.clone();
    }

    style
}

/// Palette and scale last applied to a context, stored so `ensure` can
/// detect a scale change and `palette` can report the active palette.
#[derive(Clone)]
struct State {
    palette: Palette,
    ppp: f32,
}

fn state_id() -> Id {
    Id::new("egui_pc98_revival::state")
}

/// Installs the bundled font (feature `bundled-font`) and the default palette.
///
/// Without the `bundled-font` feature, call
/// [`crate::fonts::font_definitions_with`] and `ctx.set_fonts(..)` yourself
/// before or after this call to install a substitute dot font.
pub fn apply(ctx: &egui::Context) {
    #[cfg(feature = "bundled-font")]
    crate::fonts::install(ctx);
    apply_with(ctx, &Palette::default());
}

/// Applies the style for `palette` at the current scale. Does not touch fonts.
pub fn apply_with(ctx: &egui::Context, palette: &Palette) {
    let ppp = ctx.pixels_per_point();
    let style = pc98_style(palette, ppp);

    // `set_visuals`/`set_style` only change the *current* theme; `set_theme` plus
    // `all_styles_mut` also covers a light-mode OS, which would otherwise fall
    // back to egui's light style.
    ctx.set_theme(ThemePreference::Dark);
    ctx.all_styles_mut(|s| *s = style.clone());

    // 1-dot lines and fills must not be anti-aliased.
    ctx.tessellation_options_mut(|t| t.feathering = false);

    ctx.data_mut(|d| {
        d.insert_temp(
            state_id(),
            State {
                palette: *palette,
                ppp,
            },
        )
    });
}

/// Cheap per-frame hook: re-applies the stored palette only when `pixels_per_point` changed.
pub fn ensure(ctx: &egui::Context) {
    let ppp = ctx.pixels_per_point();
    let state: Option<State> = ctx.data(|d| d.get_temp(state_id()));
    let palette = match state {
        Some(state) if state.ppp == ppp => return,
        Some(state) => state.palette,
        None => Palette::default(),
    };
    apply_with(ctx, &palette);
    // The root `Ui` of this pass already copied the old style, so redo the pass
    // rather than leaving a stale frame on screen until the next input event.
    ctx.request_discard("egui_pc98_revival: style re-applied");
}

/// Palette last applied to this context, or the default one.
pub fn palette(ctx: &egui::Context) -> Palette {
    ctx.data(|d| d.get_temp::<State>(state_id()))
        .map(|s| s.palette)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_table() {
        let cases: [(f32, f32, f32); 5] = [
            (1.0, 1.0, 16.0),
            (1.5, 4.0 / 3.0, 64.0 / 3.0),
            (2.0, 1.0, 16.0),
            (1.25, 0.8, 12.8),
            (0.5, 2.0, 32.0),
        ];
        for (ppp, expected_dot, expected_font) in cases {
            assert!(
                (dot_for_ppp(ppp) - expected_dot).abs() < 1e-6,
                "dot_for_ppp({ppp}) = {}, expected {expected_dot}",
                dot_for_ppp(ppp)
            );
            assert!(
                (font_size_for_ppp(ppp) - expected_font).abs() < 1e-6,
                "font_size_for_ppp({ppp}) = {}, expected {expected_font}",
                font_size_for_ppp(ppp)
            );
        }
    }

    #[test]
    fn pc98_style_text_styles_are_monospace_16dot() {
        let style = pc98_style(&Palette::PC98, 1.5);
        let expected = FontId::new(64.0 / 3.0, FontFamily::Monospace);
        for font_id in style.text_styles.values() {
            assert_eq!(*font_id, expected);
        }
    }

    #[test]
    fn pc98_style_override_text_color_is_none() {
        let style = pc98_style(&Palette::PC98, 1.5);
        assert_eq!(style.visuals.override_text_color, None);
    }

    #[test]
    fn pc98_style_hovered_uses_accent() {
        let style = pc98_style(&Palette::PC98, 1.5);
        assert_eq!(
            style.visuals.widgets.hovered.weak_bg_fill,
            Palette::PC98.accent
        );
    }

    #[test]
    fn pc98_style_hovered_text_uses_hover_fg() {
        let style = pc98_style(&Palette::PC98, 1.5);
        assert_eq!(
            style.visuals.widgets.hovered.fg_stroke.color,
            Palette::PC98.hover_fg
        );
        assert_eq!(
            style.visuals.widgets.active.fg_stroke.color,
            Palette::PC98.hover_fg
        );
    }

    #[test]
    fn pc98_style_widget_states_have_no_corner_radius() {
        let style = pc98_style(&Palette::PC98, 1.5);
        let w = &style.visuals.widgets;
        for state in [
            &w.noninteractive,
            &w.inactive,
            &w.hovered,
            &w.active,
            &w.open,
        ] {
            assert_eq!(state.corner_radius, CornerRadius::ZERO);
        }
    }

    #[test]
    fn pc98_style_window_shadow_is_none() {
        let style = pc98_style(&Palette::PC98, 1.5);
        assert_eq!(style.visuals.window_shadow, Shadow::NONE);
    }

    #[test]
    fn pc98_style_noninteractive_bg_stroke_width() {
        let style = pc98_style(&Palette::PC98, 1.5);
        assert!((style.visuals.widgets.noninteractive.bg_stroke.width - 4.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn apply_with_sets_no_shadow_and_no_feathering() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());
        assert_eq!(
            ctx.style_of(ctx.theme()).visuals.window_shadow,
            Shadow::NONE
        );
        assert!(!ctx.tessellation_options(|t| t.feathering));
    }

    #[test]
    fn ensure_does_not_panic_when_called_repeatedly() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());
        ensure(&ctx);
        ensure(&ctx);
    }

    #[test]
    fn ensure_applies_default_palette_on_fresh_context() {
        let ctx = egui::Context::default();
        ensure(&ctx);
        assert!(!ctx.tessellation_options(|t| t.feathering));
        assert_eq!(palette(&ctx), Palette::PC98);
    }

    #[test]
    fn dots_scales_with_dot() {
        let ctx = egui::Context::default();
        assert_eq!(dots(&ctx, 4.0), 4.0);
    }

    #[test]
    fn snap_rect_rounds_to_device_pixels() {
        let ppp = 1.5;
        let rect = Rect::from_min_max(pos2(0.2, 0.9), pos2(10.1, 10.4));
        let snapped = snap_rect_for_ppp(rect, ppp);
        let step = 1.0 / ppp;
        for (snapped_v, original_v) in [
            (snapped.min.x, rect.min.x),
            (snapped.min.y, rect.min.y),
            (snapped.max.x, rect.max.x),
            (snapped.max.y, rect.max.y),
        ] {
            let n = snapped_v / step;
            assert!(
                (n - n.round()).abs() < 1e-4,
                "{snapped_v} is not a multiple of {step}"
            );
            assert!((snapped_v - original_v).abs() <= 0.5 * step + 1e-4);
        }
    }
}
