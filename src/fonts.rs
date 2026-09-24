//! The bundled 16-dot bitmap font and helpers to install it or a substitute.

use std::borrow::Cow;
use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily, FontTweak};

/// Name the bundled font is registered under.
pub const FONT_NAME: &str = "KH-Dot-Kodenmachou-16";

#[cfg(feature = "bundled-font")]
pub const KH_DOT_16_TTF: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/KH-Dot-Kodenmachou-16-Ki.ttf"
));

/// Wraps raw TTF bytes with tweaks suited to dot fonts.
///
/// Hinting would re-fit the square, pixel-grid-aligned glyph outlines, and
/// sub-pixel binning would smear them; both are disabled.
pub fn pixel_font_data(bytes: impl Into<Cow<'static, [u8]>>) -> FontData {
    FontData {
        font: bytes.into(),
        index: 0,
        tweak: FontTweak {
            hinting: Some(false),
            subpixel_binning: Some(false),
            ..Default::default()
        },
    }
}

/// egui defaults with `data` inserted first in both the Monospace and
/// Proportional families.
pub fn font_definitions_with(name: &str, data: FontData) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(name.to_owned(), Arc::new(data));
    for family in [FontFamily::Monospace, FontFamily::Proportional] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, name.to_owned());
    }
    fonts
}

/// [`font_definitions_with`] for the bundled font.
#[cfg(feature = "bundled-font")]
pub fn font_definitions() -> FontDefinitions {
    font_definitions_with(FONT_NAME, pixel_font_data(KH_DOT_16_TTF))
}

/// Installs the bundled font on `ctx`.
#[cfg(feature = "bundled-font")]
pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(font_definitions());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_definitions_with_inserts_at_front_and_keeps_defaults() {
        let fonts = font_definitions_with("x", pixel_font_data(&[0u8; 4][..]));
        let defaults = FontDefinitions::default();
        for family in [FontFamily::Monospace, FontFamily::Proportional] {
            let names = &fonts.families[&family];
            assert_eq!(names[0], "x");
            assert_eq!(&names[1..], &defaults.families[&family][..]);
        }
        assert!(fonts.font_data.contains_key("x"));
    }

    #[cfg(feature = "bundled-font")]
    #[test]
    fn bundled_font_definitions_contain_the_font() {
        let fonts = font_definitions();
        for family in [FontFamily::Monospace, FontFamily::Proportional] {
            assert_eq!(fonts.families[&family][0], FONT_NAME);
        }
        assert!(KH_DOT_16_TTF.len() > 1_000_000);
    }
}
