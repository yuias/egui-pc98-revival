//! Modal titled panel centered over a dithered scrim.

use egui::{Color32, Context, Id, Key, Modifiers, Response, Ui, Vec2, vec2};

use crate::widgets::{TitledPanel, paint_scrim};

/// Modal titled panel centered over a dithered scrim. The background does not
/// take input; Esc requests close.
pub struct Dialog<'a> {
    id: Id,
    title: &'a str,
    size: Option<Vec2>,
    close_on_backdrop_click: bool,
}

impl<'a> Dialog<'a> {
    /// New dialog identified by `id`, titled `title`. `close_on_backdrop_click` starts false.
    pub fn new(id: Id, title: &'a str) -> Self {
        Self {
            id,
            title,
            size: None,
            close_on_backdrop_click: false,
        }
    }

    /// Outer size including title strip; default is 60% of the content rect.
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }

    /// Whether a click on the backdrop, outside the panel, requests close.
    pub fn close_on_backdrop_click(mut self, close: bool) -> Self {
        self.close_on_backdrop_click = close;
        self
    }

    /// Shows the dialog as a modal over the whole screen.
    pub fn show<R>(
        self,
        ctx: &Context,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> DialogResponse<R> {
        let ppp = ctx.pixels_per_point();
        let raw_size = self.size.unwrap_or_else(|| ctx.content_rect().size() * 0.6);
        // Snapped like `snap_rect`, so the panel's edges land on device pixels.
        let size = vec2(
            (raw_size.x * ppp).round() / ppp,
            (raw_size.y * ppp).round() / ppp,
        );
        let title = self.title;
        let close_on_backdrop_click = self.close_on_backdrop_click;

        let modal = egui::Modal::new(self.id)
            .backdrop_color(Color32::TRANSPARENT)
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                // Painted first in the modal's own layer so the panel below draws over it.
                let screen = ui.ctx().content_rect();
                paint_scrim(&ui.painter().with_clip_rect(screen), screen);
                ui.allocate_ui(size, |ui| {
                    ui.set_min_size(size);
                    TitledPanel::new(title).show(ui, add_contents).inner
                })
                .inner
            });

        // Consumed only when this modal is topmost, matching `ModalResponse::should_close`'s
        // own escape handling but without also treating a plain backdrop click as close.
        let esc = modal.is_top_modal
            && !modal.any_popup_open
            && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        let close_requested = esc
            || modal.response.should_close()
            || (close_on_backdrop_click && modal.backdrop_response.clicked());

        DialogResponse {
            inner: modal.inner,
            close_requested,
            response: modal.response,
        }
    }
}

/// Result of showing a [`Dialog`].
pub struct DialogResponse<R> {
    pub inner: R,
    /// Esc (when topmost and no popup is open), `ui.close()` inside the
    /// dialog, or a backdrop click if enabled. The caller stops showing the dialog.
    pub close_requested: bool,
    /// Response of the dialog area.
    pub response: Response,
}

#[cfg(test)]
mod tests {
    use egui::{Event, Key as EguiKey, Modifiers as EguiModifiers, RawInput, pos2};

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    #[test]
    fn dialog_smoke_returns_inner() {
        let ctx = Context::default();
        apply_with(&ctx, &Palette::default());

        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let result = Dialog::new(Id::new("d"), "T").show(ui.ctx(), |ui| {
                ui.label("hi");
                42
            });
            assert_eq!(result.inner, 42);
            assert!(!result.close_requested);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn dialog_escape_requests_close() {
        let ctx = Context::default();
        apply_with(&ctx, &Palette::default());

        // Warm-up frames: hit-testing (and the modal's `is_top_modal` bookkeeping)
        // uses the previous pass's state.
        for _ in 0..2 {
            let mut output = ctx.run_ui(RawInput::default(), |ui| {
                Dialog::new(Id::new("d"), "T").show(ui.ctx(), |ui| {
                    ui.label("hi");
                });
            });
            output.textures_delta.clear();
        }

        let input = RawInput {
            events: vec![Event::Key {
                key: EguiKey::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: EguiModifiers::NONE,
            }],
            ..Default::default()
        };
        let mut close_requested = false;
        let mut output = ctx.run_ui(input, |ui| {
            let result = Dialog::new(Id::new("d"), "T").show(ui.ctx(), |ui| {
                ui.label("hi");
            });
            close_requested = result.close_requested;
        });
        output.textures_delta.clear();

        assert!(close_requested);
    }

    #[test]
    fn dialog_backdrop_click_ignored_by_default() {
        let ctx = Context::default();
        apply_with(&ctx, &Palette::default());

        for _ in 0..2 {
            let mut output = ctx.run_ui(RawInput::default(), |ui| {
                Dialog::new(Id::new("d"), "T").show(ui.ctx(), |ui| {
                    ui.label("hi");
                });
            });
            output.textures_delta.clear();
        }

        // Screen corner: far from the centred panel, so it lands on the backdrop.
        let corner = pos2(1.0, 1.0);
        let mut close_requested = true;
        for pressed in [true, false] {
            let input = RawInput {
                events: vec![
                    Event::PointerMoved(corner),
                    Event::PointerButton {
                        pos: corner,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: EguiModifiers::NONE,
                    },
                ],
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| {
                let result = Dialog::new(Id::new("d"), "T").show(ui.ctx(), |ui| {
                    ui.label("hi");
                });
                close_requested = result.close_requested;
            });
            output.textures_delta.clear();
        }

        assert!(!close_requested);
    }
}
