//! The digital 8-color PC-98 palette and the `Palette` struct widgets read
//! their colors from.

use egui::Color32;

pub const GROUND: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
pub const BAR_BG: Color32 = Color32::from_rgb(0x22, 0x33, 0xCC);
pub const BAR_FG: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const FRAME: Color32 = Color32::from_rgb(0x22, 0xCC, 0xCC);
pub const TITLE_BG: Color32 = Color32::from_rgb(0x22, 0xCC, 0xCC);
pub const TITLE_FG: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
pub const TEXT: Color32 = Color32::from_rgb(0xEE, 0xEE, 0xEE);
pub const DIM: Color32 = Color32::from_rgb(0x88, 0x88, 0x88);
pub const ACCENT: Color32 = Color32::from_rgb(0xEE, 0xDD, 0x22);
pub const RED: Color32 = Color32::from_rgb(0xEE, 0x22, 0x33);
pub const PLAYHEAD: Color32 = RED;
pub const SELECTED_BG: Color32 = Color32::from_rgb(0x22, 0x33, 0xCC);
pub const WELL: Color32 = Color32::from_rgb(0x16, 0x16, 0x16);
pub const GREEN: Color32 = Color32::from_rgb(0x22, 0xCC, 0x44);
pub const OK: Color32 = GREEN;
pub const FKEY_BG: Color32 = Color32::from_rgb(0xEE, 0xEE, 0xEE);
pub const FKEY_FG: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
pub const WARN: Color32 = ACCENT;
pub const DANGER: Color32 = RED;
pub const SELECTED_FG: Color32 = BAR_FG;
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
    pub ground: Color32,
    pub bar_bg: Color32,
    pub bar_fg: Color32,
    pub frame: Color32,
    pub title_bg: Color32,
    pub title_fg: Color32,
    pub text: Color32,
    pub dim: Color32,
    pub accent: Color32,
    pub red: Color32,
    pub selected_bg: Color32,
    pub well: Color32,
    pub green: Color32,
    pub fkey_bg: Color32,
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
