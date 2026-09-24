//! Modal `Dialog` with a text body and an fkey-style button row.

use egui::{TextStyle, vec2};

use crate::style::dots;
use crate::text::{cell_width, cells, truncate_tail};
use crate::widgets::{Dialog, FKey, fkey_bar};

/// How a message box was closed this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageBoxResult {
    /// Button `i` was clicked or its shortcut pressed.
    Button(usize),
    /// Esc (with no button bound to Esc) or `ui.close()`.
    Dismissed,
}

/// Modal `Dialog` with `text` and an fkey-style button row. Returns `Some`
/// on the frame the user decides; the caller then stops calling it.
pub fn message_box(
    ctx: &egui::Context,
    id: egui::Id,
    title: &str,
    text: &str,
    buttons: &[FKey<'_>],
) -> Option<MessageBoxResult> {
    let font_id = TextStyle::Body.resolve(&ctx.style_of(ctx.theme()));
    // `glyph_width`/`row_height` need `&mut Fonts`, unlike the read-only `fonts()` accessor.
    let (cell_w, row_h) = ctx.fonts_mut(|f| (f.glyph_width(&font_id, '0'), f.row_height(&font_id)));

    let lines: Vec<&str> = text.lines().collect();
    let text_cells = lines.iter().map(|l| cells(l)).max().unwrap_or(0);
    let key_cells = buttons.iter().map(|b| cells(b.key)).max().unwrap_or(0);
    let label_cells = buttons.iter().map(|b| cells(b.label)).max().unwrap_or(0);
    // Approximation of `fit_fkey_items`' need: key + label + 3 insets of 4 dots ~ 1.5 cells each.
    let button_cells = buttons.len() * (key_cells + label_cells + 2);

    let inner_cells = [text_cells + 2, button_cells, cells(title) + 4, 20]
        .into_iter()
        .max()
        .unwrap_or(20);
    let inner_width = inner_cells as f32 * cell_w;
    let outer_width = (inner_width + dots(ctx, 8.0)).min(ctx.content_rect().width() * 0.9);

    let title_h = row_h + dots(ctx, 4.0);
    let fkey_h = row_h + dots(ctx, 8.0);
    let content_h = dots(ctx, 8.0) + (lines.len() as f32 + 1.0) * row_h;
    let outer_height = (title_h + content_h + fkey_h).min(ctx.content_rect().height() * 0.9);

    let mut button_result = None;
    let dialog = Dialog::new(id, title)
        .size(vec2(outer_width, outer_height))
        .show(ctx, |ui| {
            for line in &lines {
                let avail_cells = (ui.available_width() / cell_width(ui)).floor().max(0.0) as usize;
                ui.label(truncate_tail(line, avail_cells));
            }
            let remaining = ui.available_height() - fkey_h;
            ui.add_space(remaining.max(0.0));
            button_result = fkey_bar(ui, buttons).map(MessageBoxResult::Button);
        });

    // A button bound to Escape wins: `fkey_bar` (inside `add_contents`, above)
    // consumes the key before `Dialog::show` checks it for the dismiss path.
    button_result.or(dialog
        .close_requested
        .then_some(MessageBoxResult::Dismissed))
}

#[cfg(test)]
mod tests {
    use egui::{Event, Key, Modifiers, RawInput};

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    fn key_event(key: Key) -> RawInput {
        RawInput {
            events: vec![Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        }
    }

    fn buttons() -> Vec<FKey<'static>> {
        vec![
            FKey::new("Y", "Yes").key_shortcut(Key::Y),
            FKey::new("N", "No").key_shortcut(Key::N),
        ]
    }

    #[test]
    fn message_box_no_input_returns_none() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());
        let buttons = buttons();

        let mut result = None;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            result = message_box(ui.ctx(), egui::Id::new("mb"), "T", "Sure?", &buttons);
        });
        output.textures_delta.clear();

        assert_eq!(result, None);
    }

    #[test]
    fn message_box_shortcut_picks_button() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());
        let buttons = buttons();

        let run_frame =
            |ctx: &egui::Context, input: RawInput, result: &mut Option<MessageBoxResult>| {
                let mut output = ctx.run_ui(input, |ui| {
                    *result = message_box(ui.ctx(), egui::Id::new("mb"), "T", "Sure?", &buttons);
                });
                output.textures_delta.clear();
            };

        let mut result = None;
        for _ in 0..2 {
            run_frame(&ctx, RawInput::default(), &mut result);
        }
        run_frame(&ctx, key_event(Key::N), &mut result);

        assert_eq!(result, Some(MessageBoxResult::Button(1)));
    }

    #[test]
    fn message_box_escape_dismisses() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());
        let buttons = buttons();

        let run_frame =
            |ctx: &egui::Context, input: RawInput, result: &mut Option<MessageBoxResult>| {
                let mut output = ctx.run_ui(input, |ui| {
                    *result = message_box(ui.ctx(), egui::Id::new("mb"), "T", "Sure?", &buttons);
                });
                output.textures_delta.clear();
            };

        let mut result = None;
        for _ in 0..2 {
            run_frame(&ctx, RawInput::default(), &mut result);
        }
        run_frame(&ctx, key_event(Key::Escape), &mut result);

        assert_eq!(result, Some(MessageBoxResult::Dismissed));
    }

    #[test]
    fn message_box_escape_bound_button_wins() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());
        let mut buttons = buttons();
        buttons.push(FKey::new("ESC", "Cancel").key_shortcut(Key::Escape));

        let run_frame =
            |ctx: &egui::Context, input: RawInput, result: &mut Option<MessageBoxResult>| {
                let mut output = ctx.run_ui(input, |ui| {
                    *result = message_box(ui.ctx(), egui::Id::new("mb"), "T", "Sure?", &buttons);
                });
                output.textures_delta.clear();
            };

        let mut result = None;
        for _ in 0..2 {
            run_frame(&ctx, RawInput::default(), &mut result);
        }
        run_frame(&ctx, key_event(Key::Escape), &mut result);

        assert_eq!(result, Some(MessageBoxResult::Button(2)));
    }
}
