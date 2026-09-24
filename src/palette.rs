//! The digital 8-color PC-98 palette and the `Palette` struct widgets read
//! their colors from.

use egui::Color32;

/// Background of the screen and window bodies.
pub const GROUND: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
/// Background of title/status bars and the selection bar.
pub const BAR_BG: Color32 = Color32::from_rgb(0x22, 0x33, 0xCC);
/// Text on `BAR_BG`.
pub const BAR_FG: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
/// 1-dot frame and inactive-widget border color.
pub const FRAME: Color32 = Color32::from_rgb(0x22, 0xCC, 0xCC);
/// Background of a `TitledPanel`'s title strip.
pub const TITLE_BG: Color32 = Color32::from_rgb(0x22, 0xCC, 0xCC);
/// Text on `TITLE_BG`.
pub const TITLE_FG: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
/// Default body text color.
pub const TEXT: Color32 = Color32::from_rgb(0xEE, 0xEE, 0xEE);
/// Muted text and disabled-widget color.
pub const DIM: Color32 = Color32::from_rgb(0x88, 0x88, 0x88);
/// Hover fill and highlight color.
pub const ACCENT: Color32 = Color32::from_rgb(0xEE, 0xDD, 0x22);
/// Base red hue.
pub const RED: Color32 = Color32::from_rgb(0xEE, 0x22, 0x33);
/// Seek bar / timeline cursor color.
pub const PLAYHEAD: Color32 = RED;
/// Background of a selected row or item.
pub const SELECTED_BG: Color32 = Color32::from_rgb(0x22, 0x33, 0xCC);
/// Background of recessed areas (scroll rail, text edit).
pub const WELL: Color32 = Color32::from_rgb(0x16, 0x16, 0x16);
/// Base green hue.
pub const GREEN: Color32 = Color32::from_rgb(0x22, 0xCC, 0x44);
/// Positive-state color; alias of `GREEN`.
pub const OK: Color32 = GREEN;
/// Background of a function-key label.
pub const FKEY_BG: Color32 = Color32::from_rgb(0xEE, 0xEE, 0xEE);
/// Text on `FKEY_BG`.
pub const FKEY_FG: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
/// Caution-state color; alias of `ACCENT`.
pub const WARN: Color32 = ACCENT;
/// Error/mute-state color; alias of `RED`.
pub const DANGER: Color32 = RED;
/// Text on `SELECTED_BG`; alias of `BAR_FG`.
pub const SELECTED_FG: Color32 = BAR_FG;
/// Text on a hovered (accent-filled) item.
pub const HOVER_FG: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
/// Red, cyan, green, yellow, magenta, blue, white, gray.
pub const HUES: [Color32; 8] = [
    Color32::from_rgb(0xEE, 0x22, 0x33),
    Color32::from_rgb(0x22, 0xCC, 0xCC),
    Color32::from_rgb(0x22, 0xCC, 0x44),
    Color32::from_rgb(0xEE, 0xDD, 0x22),
    Color32::from_rgb(0xCC, 0x33, 0xCC),
    Color32::from_rgb(0x55, 0x66, 0xFF),
    Color32::from_rgb(0xEE, 0xEE, 0xEE),
    Color32::from_rgb(0x99, 0x99, 0x99),
];

/// The set of colors a PC-98 style is built from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    /// Background of the screen and window bodies.
    pub ground: Color32,
    /// Background of title/status bars and the selection bar.
    pub bar_bg: Color32,
    /// Text on `bar_bg`.
    pub bar_fg: Color32,
    /// 1-dot frame and inactive-widget border color.
    pub frame: Color32,
    /// Background of a `TitledPanel`'s title strip.
    pub title_bg: Color32,
    /// Text on `title_bg`.
    pub title_fg: Color32,
    /// Default body text color.
    pub text: Color32,
    /// Muted text and disabled-widget color.
    pub dim: Color32,
    /// Hover fill and highlight color.
    pub accent: Color32,
    /// Base red hue.
    pub red: Color32,
    /// Background of a selected row or item.
    pub selected_bg: Color32,
    /// Background of recessed areas (scroll rail, text edit).
    pub well: Color32,
    /// Base green hue.
    pub green: Color32,
    /// Background of a function-key label.
    pub fkey_bg: Color32,
    /// Text on `fkey_bg`.
    pub fkey_fg: Color32,
    /// Positive state (e.g. "done", "on air").
    pub ok: Color32,
    /// Caution: meter top segments, warnings.
    pub warn: Color32,
    /// Errors, mute, playhead.
    pub danger: Color32,
    /// Text on `selected_bg`.
    pub selected_fg: Color32,
    /// Text on a hovered (accent-filled) item.
    pub hover_fg: Color32,
    /// Red, cyan, green, yellow, magenta, blue, white, gray.
    pub hues: [Color32; 8],
}

impl Palette {
    /// The stock NEC PC-98 palette.
    pub const PC98: Palette = Palette {
        ground: GROUND,
        bar_bg: BAR_BG,
        bar_fg: BAR_FG,
        frame: FRAME,
        title_bg: TITLE_BG,
        title_fg: TITLE_FG,
        text: TEXT,
        dim: DIM,
        accent: ACCENT,
        red: RED,
        selected_bg: SELECTED_BG,
        well: WELL,
        green: GREEN,
        fkey_bg: FKEY_BG,
        fkey_fg: FKEY_FG,
        ok: OK,
        warn: WARN,
        danger: DANGER,
        selected_fg: SELECTED_FG,
        hover_fg: HOVER_FG,
        hues: HUES,
    };

    /// Hue `i` of the 8-color set, wrapping around.
    pub fn hue(&self, i: usize) -> Color32 {
        self.hues[i % self.hues.len()]
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::PC98
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_pc98() {
        assert_eq!(Palette::default(), Palette::PC98);
    }

    #[test]
    fn pc98_accent_matches_const() {
        assert_eq!(Palette::PC98.accent, ACCENT);
    }

    #[test]
    fn first_hue_is_red() {
        assert_eq!(HUES[0], RED);
    }

    #[test]
    fn pc98_semantic_roles_match_consts() {
        let p = Palette::PC98;
        assert_eq!(p.ok, OK);
        assert_eq!(p.warn, WARN);
        assert_eq!(p.danger, DANGER);
        assert_eq!(p.selected_fg, SELECTED_FG);
        assert_eq!(p.hover_fg, HOVER_FG);
    }

    #[test]
    fn hue_wraps() {
        let p = Palette::PC98;
        assert_eq!(p.hue(8), p.hues[0]);
    }
}
