//! A theme, bundled bitmap font, and small set of widgets that make an
//! [egui](https://docs.rs/egui) app look like a NEC PC-98 text-mode UI.
//!
//! The look: an 8-color palette, 1-dot frames, no anti-aliasing, geometry
//! snapped to the device pixel (dot) grid, and a bundled 16-dot bitmap font
//! (KH Dot Kodenmachou 16, feature `bundled-font`, enabled by default).
//!
//! # Quick start
//!
//! ```
//! use egui::RawInput;
//! use egui_pc98_revival::{FKey, apply, fkey_bar, panel};
//!
//! let ctx = egui::Context::default();
//! apply(&ctx);
//!
//! let mut output = ctx.run_ui(RawInput::default(), |ui| {
//!     panel(ui, "Status", |ui| {
//!         ui.label("Ready");
//!     });
//!     fkey_bar(ui, &[FKey::new("F1", "Help")]);
//! });
//! // Laying out text rasterizes glyphs into the font atlas; the resulting
//! // texture delta must be consumed (normally by a renderer) or applied.
//! output.textures_delta.clear();
//! ```
//!
//! Call [`ensure`] once per frame (after `apply`) to keep the style in sync
//! when `pixels_per_point` changes, e.g. the OS display scale.
//!
//! # Glyph caveats
//!
//! The bundled font follows JIS X 0208 and departs from a plain Unicode
//! rendering at a few code points: backslash renders as the yen sign, the
//! fullwidth hyphen-minus (U+FF0D) is missing (use the minus sign U+2212
//! instead), and box-drawing characters and `▶` are absent. Unicode
//! Ambiguous-width characters are drawn full-width, matching the font's
//! East Asian layout (see [`text::char_cells`]).
//!
//! # API index
//!
//! ## Setup & style
//! [`apply`], [`apply_with`], [`ensure`], [`palette()`], [`dot`], [`dots`],
//! [`snap_rect`], [`font_id_scaled`], [`Palette`], the [`fonts`] module.
//!
//! ## Layout & chrome
//! [`TitledPanel`]/[`panel`], [`HeaderBar`]/[`header_bar`],
//! [`fkey_bar`]/[`FKey`], [`tab_strip`], [`Dialog`], [`message_box`],
//! [`key_help`].
//!
//! ## Controls
//! [`ListView`] and its supporting types ([`Column`], [`ColumnWidth`],
//! [`ListRow`], [`ListState`], [`ListResponse`]), [`SegmentBar`],
//! [`seek_bar`], [`toggle_box`], [`text_checkbox`]/[`text_radio`],
//! [`icon_button`].
//!
//! ## Display
//! [`marquee`], [`text_spinner`].
//!
//! ## Painting
//! [`paint_frame`], [`paint_double_frame`], [`paint_dither`], [`paint_scrim`],
//! [`paint_hue_fill`], [`paint_dot_icon`]/[`DotIcon`].
//!
//! ## Text & layout helpers
//! the [`text`] module, [`fit_by_priority`].

#![warn(missing_docs)]

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
    icon_button, key_help, marquee, message_box, paint_dither, paint_dot_icon, paint_double_frame,
    paint_frame, paint_hue_fill, paint_scrim, panel, seek_bar, tab_strip, text_checkbox,
    text_radio, text_spinner, toggle_box, toggle_box_colored,
};
