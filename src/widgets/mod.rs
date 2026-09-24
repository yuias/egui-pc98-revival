//! PC-98 widgets built on top of [`crate::style`].

mod check;
mod dialog;
mod dither;
mod fkey;
mod frame;
mod header;
mod icon;
mod key_help;
mod list;
mod message_box;
mod panel;
mod seek;
mod segment;
mod tabs;
mod toggle;

pub use check::{text_checkbox, text_radio};
pub use dialog::{Dialog, DialogResponse};
pub use dither::{paint_dither, paint_hue_fill, paint_scrim};
pub use fkey::{FKey, fkey_bar};
pub use frame::{paint_double_frame, paint_frame};
pub use header::{HeaderBar, header_bar};
pub use icon::{DotIcon, icon_button, paint_dot_icon};
pub use key_help::key_help;
pub use list::{Column, ColumnWidth, ListResponse, ListRow, ListState, ListView};
pub use message_box::{MessageBoxResult, message_box};
pub use panel::{TitledPanel, panel};
pub use seek::seek_bar;
pub use segment::SegmentBar;
pub use tabs::{TabStyle, tab_strip};
pub use toggle::{toggle_box, toggle_box_colored};
