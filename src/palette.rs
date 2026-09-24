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
        hues: HUES,
    };
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
}
