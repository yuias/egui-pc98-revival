//! A theme, bundled bitmap font and small set of widgets that make an egui
//! app look like a NEC PC-98 text-mode UI.

pub mod fonts;
pub mod layout;
pub mod palette;
pub mod style;
pub mod text;
pub mod widgets;

pub use layout::fit_by_priority;
pub use palette::Palette;
pub use style::{apply, apply_with, dot, dots, ensure, palette, snap_rect};
pub use widgets::{
    Column, ColumnWidth, FKey, ListRow, ListState, SegmentBar, TabStyle, TitledPanel, fkey_bar,
    header_bar, paint_dither, paint_frame, paint_scrim, panel, tab_strip,
};
