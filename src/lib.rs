//! A theme, bundled bitmap font and small set of widgets that make an egui
//! app look like a NEC PC-98 text-mode UI.

pub mod fonts;
pub mod palette;
pub mod style;

pub use palette::Palette;
pub use style::{apply, apply_with, ensure};
