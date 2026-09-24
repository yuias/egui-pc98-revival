//! PC-98 widgets built on top of [`crate::style`].

mod dither;
mod fkey;
mod header;
mod panel;

pub use dither::paint_dither;
pub use fkey::{FKey, fkey_bar};
pub use header::header_bar;
pub use panel::panel;
