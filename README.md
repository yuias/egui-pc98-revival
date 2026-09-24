# egui-pc98-revival

A theme, bundled bitmap font, and small set of widgets that make an
[egui](https://github.com/emilk/egui) app look like a NEC PC-98 text-mode UI:
a digital 8-color palette, sharp corners with no shadows or anti-aliasing,
line widths and font size snapped to whole device pixels, and widgets for a
titled panel, a header bar, a function-key bar, and a 1-dot dither fill.

## Usage

```toml
[dependencies]
egui-pc98-revival = "0.1"
```

```rust
// Once, when the egui context is created:
egui_pc98_revival::apply(&cc.egui_ctx);

// Every frame, before drawing:
egui_pc98_revival::ensure(ctx);

// A titled panel with a function-key bar below it:
egui_pc98_revival::TitledPanel::new("FILES")
    .focused(true)
    .show_with_title(
        ui,
        |ui| { ui.label("25 FILES"); }, // right-aligned slot in the title strip
        |ui| { ui.label("some content"); },
    );
// `panel(ui, title, add_contents)` is shorthand for `TitledPanel::new(title).show(..)`.
let items = [
    egui_pc98_revival::FKey::new("F1", "Help").key_shortcut(egui::Key::F1),
    egui_pc98_revival::FKey::new("F10", "Quit").key_shortcut(egui::Key::F10),
];
if let Some(clicked) = egui_pc98_revival::fkey_bar(ui, &items) {
    // handle a click on items[clicked], or its bound key having been pressed
}
// A disabled slot (`.enabled(false)`) draws dimmed and ignores both.
```

`apply` forces the dark theme, since the PC-98 look has no light variant.

### Custom widgets

Build your own widgets on the same dot grid and palette as the bundled ones:

```rust
let d = egui_pc98_revival::dot(ui.ctx());       // 1 dot, in points
let inset = egui_pc98_revival::dots(ui.ctx(), 4.0); // n dots, in points
let palette = egui_pc98_revival::palette(ui.ctx());

let rect = egui_pc98_revival::snap_rect(ui.ctx(), ui.available_rect_before_wrap());
ui.painter().rect_filled(rect.shrink(inset), 0, palette.ground);
egui_pc98_revival::paint_frame(ui.painter(), rect, palette.frame);
```

`Palette` also carries semantic roles (`ok`, `warn`, `danger`, `selected_fg`,
`hover_fg`) so status colors stay consistent with the active theme.

`fit_by_priority` decides which items to drop when a row of widths does not
fit the available space, dropping the highest-priority droppable item first:

```rust
let widths = [40.0, 30.0, 20.0];
let drop_priority = [None, Some(1), Some(2)]; // first item is never dropped
let keep = egui_pc98_revival::fit_by_priority(&widths, &drop_priority, 4.0, 75.0);
```

### Text cells

`text::cells` and friends count East Asian Wide, Fullwidth and Ambiguous
characters as 2 cells and everything else as 1, with overrides where the
bundled font differs (it draws Greek, Cyrillic and `−` full-width), so the
count matches how the bundled font renders text (box drawing and `▶` are not in the bundled font and fall
back to egui's default font, so widget-produced text stays ASCII):

```rust
use egui_pc98_revival::text;

assert_eq!(text::cells("日本語"), 6);
assert_eq!(text::truncate_tail("日本電気株式会社製パーソナルコンピュータ", 12), "日本電気...");
let cell_w = text::cell_width(ui);
```

`truncate_tail` truncates to whole cells and appends an ASCII `"..."`
(never `…`) when it cuts text short.

### Tab strip

`tab_strip` draws a row of tabs and updates `selected` when one is clicked;
`TabStyle::TitleStrip` is meant for a `TitledPanel` title-strip slot,
`TabStyle::Bar` for a standalone strip on `ground`:

```rust
egui_pc98_revival::tab_strip(
    ui,
    &mut self.tab,
    &["GENERAL", "SOUND"],
    egui_pc98_revival::TabStyle::Bar,
);
```

### Segment bar

`SegmentBar` draws a row or column of equal blocks with 1-dot gaps (level
meter, volume blocks, gauge). `show` is display-only; `show_interactive`
also reads click, drag and mouse wheel input and writes whole-segment steps
back into `value`:

```rust
egui_pc98_revival::SegmentBar::new(12, level)
    .warn(10, palette.warn) // top 2 segments turn warn-colored when lit
    .show(ui);

egui_pc98_revival::SegmentBar::new(10, 0.0)
    .fill(palette.frame)
    .warn(8, palette.accent)
    .show_interactive(ui, &mut self.volume);
```

### List view

`ListView` is a keyboard-driven scrolling list: a column header, a full-width
selection bar that follows the cursor (Up/Down/PageUp/PageDown/Home/End), and
priority-based column dropping when narrow. `state.cursor` is the persistent
selection; `add_row` draws each visible row through `ListRow::cell` (or
`cell_ui` for custom widgets); `activated` is set on Enter or a double-click:

```rust
let columns = [
    egui_pc98_revival::Column::new("NAME", egui_pc98_revival::ColumnWidth::Fill { min_cells: 12 }),
    egui_pc98_revival::Column::new("SIZE", egui_pc98_revival::ColumnWidth::Cells(7))
        .align(egui::Align::Max)
        .drop_priority(1),
];
let response = egui_pc98_revival::ListView::new("files", names.len())
    .columns(&columns)
    .show(ui, &mut self.list_state, |row, i| {
        row.cell(0, names[i], None);
        row.cell(1, &sizes[i], None);
    });
if let Some(i) = response.activated {
    // Enter or double-click on row `i`.
}
```

### Dot icons and icon button

`DotIcon` variants are glyph-free: the font draws Ambiguous symbols (○●■□
etc.) full-width, so a real icon set is drawn from axis-aligned dot rects
instead. `icon_button` is a square, framed button; `paint_dot_icon` paints an
icon directly, scaled by a whole number of dots to fit the given rect:

```rust
if egui_pc98_revival::icon_button(ui, egui_pc98_revival::DotIcon::Play).clicked() {
    // ...
}

let rect = egui::Rect::from_min_size(pos, egui::vec2(28.0, 28.0)); // 2x a 7x7 icon at 2 dots/cell
egui_pc98_revival::paint_dot_icon(
    ui.painter(),
    rect,
    egui_pc98_revival::DotIcon::Pause,
    egui_pc98_revival::palette(ui.ctx()).text,
);
```

### Dialog

`Dialog` shows a `TitledPanel` centered over a dithered scrim; the background
does not take input. Esc requests close, and so does a `ui.close()` call from
inside `add_contents`; the caller owns the flag that keeps the dialog open:

```rust
if self.show_help {
    let result = egui_pc98_revival::Dialog::new(egui::Id::new("help"), "HELP")
        .show(ui.ctx(), |ui| {
            ui.label("Esc  Close this dialog");
        });
    if result.close_requested {
        self.show_help = false;
    }
}
```

## Feature flags

- `bundled-font` (on by default): bundles a baseline-aligned copy of the
  "KH Dot Kodenmachou 16" font (see [License](#license)) and installs it as
  the default monospace and proportional font.

Without this feature, install a font yourself with
`fonts::font_definitions_with`, or set the fonts through `egui::Context` as
usual.

## Pixel snapping

One "dot" is a whole number of device pixels, computed from the current
`pixels_per_point`. Frame strokes and the font size set by
`style::pc98_style` are multiples of a dot, so they land exactly on the
device pixel grid instead of being anti-aliased. Call `ensure(ctx)` once per frame;
it re-applies the style only when `pixels_per_point` has changed, for example
after the window moves to a display with a different OS scale factor.

## Running the example

```sh
cargo run --example demo
```

## Stock widget notes

`apply` restyles egui's own widgets (buttons, menus, combo boxes, windows,
scroll areas, and so on), but `egui::ProgressBar` rounds its ends regardless
of the style. Pass `.corner_radius(0)` to keep it square.

## Full-width character note

The bundled font maps the JIS full-width minus sign to U+2212 (MINUS SIGN,
`−`), not U+FF0D (FULLWIDTH HYPHEN-MINUS, `－`). Use `−` (U+2212) in text
passed to this font; `－` (U+FF0D) renders as a missing glyph.

## License

The code in this repository is licensed under the MIT license; see
[`LICENSE`](LICENSE).

The bundled font, "KH Dot Kodenmachou 16"
(`assets/fonts/KH-Dot-Kodenmachou-16-Ki.ttf`), was designed by Keitarou
Hiraki and converted to TrueType by Jikasei Font Koubou
(<http://jikasei.me/font/kh-dotfont/>). It is licensed under the SIL Open
Font License 1.1; the full text is in
[`assets/fonts/SIL_Open_Font_License_1.1.txt`](assets/fonts/SIL_Open_Font_License_1.1.txt).

The crate embeds a modified copy,
`assets/fonts/KH-Dot-Kodenmachou-16-Ki-aligned.ttf`, which is also under the
SIL Open Font License 1.1. The original places its dot grid a quarter dot
below the baseline, so egui splits every horizontal stroke across two pixel
rows. The copy moves all outlines and vertical metrics up by that quarter dot
and is otherwise unchanged. Regenerate it with:

```sh
uv run scripts/align_font.py assets/fonts/KH-Dot-Kodenmachou-16-Ki.ttf     assets/fonts/KH-Dot-Kodenmachou-16-Ki-aligned.ttf
```
