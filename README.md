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
egui_pc98_revival::panel(ui, "FILES", |ui| {
    ui.label("some content");
});
let items = [
    egui_pc98_revival::FKey { key: "F1", label: "Help" },
    egui_pc98_revival::FKey { key: "F10", label: "Quit" },
];
if let Some(clicked) = egui_pc98_revival::fkey_bar(ui, &items) {
    // handle click on items[clicked]
}
```

`apply` forces the dark theme, since the PC-98 look has no light variant.

## Feature flags

- `bundled-font` (on by default): bundles and installs the "KH Dot
  Kodenmachou 16" font as the default monospace and proportional font.

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
