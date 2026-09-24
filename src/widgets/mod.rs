//! PC-98 widgets built on top of [`crate::style`].

mod dialog;
mod dither;
mod fkey;
mod frame;
mod header;
mod icon;
mod list;
mod panel;
mod segment;
mod tabs;

pub use dialog::{Dialog, DialogResponse};
pub use dither::{paint_dither, paint_scrim};
pub use fkey::{FKey, fkey_bar};
pub use frame::paint_frame;
pub use header::header_bar;
pub use icon::{DotIcon, icon_button, paint_dot_icon};
pub use list::{Column, ColumnWidth, ListResponse, ListRow, ListState, ListView};
pub use panel::{TitledPanel, panel};
pub use segment::SegmentBar;
pub use tabs::{TabStyle, tab_strip};
