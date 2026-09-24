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
pub use style::{apply, apply_with, dot, dots, ensure, font_id_scaled, palette, snap_rect};
pub use widgets::{
    Column, ColumnWidth, Dialog, DialogResponse, DotIcon, FKey, HeaderBar, ListResponse, ListRow,
    ListState, ListView, MessageBoxResult, SegmentBar, TabStyle, TitledPanel, fkey_bar, header_bar,
    icon_button, key_help, message_box, paint_dither, paint_dot_icon, paint_double_frame,
    paint_frame, paint_hue_fill, paint_scrim, panel, seek_bar, tab_strip, text_checkbox,
    text_radio, text_spinner, toggle_box, toggle_box_colored,
};
