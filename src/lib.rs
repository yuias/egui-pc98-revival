//! A theme, bundled bitmap font and small set of widgets that make an egui
//! app look like a NEC PC-98 text-mode UI.

pub mod fonts;
pub mod palette;
pub mod style;
pub mod widgets;

pub use palette::Palette;
pub use style::{apply, apply_with, dot, dots, ensure, palette, snap_rect};
pub use widgets::{FKey, fkey_bar, header_bar, paint_dither, paint_frame, paint_scrim, panel};
