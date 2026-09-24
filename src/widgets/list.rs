//! Keyboard-driven list view: fixed/fill columns with priority-based
//! dropping, cell drawing truncated to fit, header, selection bar, and
//! viewport-culled scrolling.

use egui::style::ScrollAnimation;
use egui::{
    Align, Color32, EventFilter, Id, Key, Layout, Modifiers, Painter, Rect, Response, ScrollArea,
    Sense, TextStyle, Ui, UiBuilder, pos2, vec2,
};

use crate::Palette;
use crate::layout::fit_by_priority;
use crate::style::palette;
use crate::text::{cell_width, truncate_tail};
use crate::widgets::paint_frame;

/// Width policy of a [`Column`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColumnWidth {
    /// Fixed width in half-width cells.
    Cells(usize),
    /// Takes the width left over; never narrower than `min_cells`.
    Fill {
        /// Minimum width, in half-width cells, before the column is dropped.
        min_cells: usize,
    },
}

/// One column of a list header/rows.
#[derive(Clone, Copy, Debug)]
pub struct Column<'a> {
    /// Header text.
    pub title: &'a str,
    /// Width policy.
    pub width: ColumnWidth,
    /// `Min` = left, `Center`, `Max` = right.
    pub align: Align,
    /// `None` = never dropped; higher values are dropped first when narrow.
    pub drop_priority: Option<u8>,
}

impl<'a> Column<'a> {
    /// New column, left-aligned, never dropped.
    pub fn new(title: &'a str, width: ColumnWidth) -> Self {
        Self {
            title,
            width,
            align: Align::Min,
            drop_priority: None,
        }
    }

    /// Sets the text alignment within the column.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Sets the drop priority: higher values are dropped first when the row is too narrow.
    pub fn drop_priority(mut self, p: u8) -> Self {
        self.drop_priority = Some(p);
        self
    }
}

/// Persistent state of a list: which row the cursor (selection bar) sits on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListState {
    /// Row under the cursor (selection bar). Clamped to the row count each frame.
    pub cursor: usize,
}

/// Horizontal placement of one visible column, relative to the row's left edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ColumnSpan {
    pub x: f32,
    pub width: f32,
    pub cells: usize,
    pub align: Align,
}

/// Places columns in `width` points: 1-cell inset at both ends, 1-cell gap
/// between columns, drops columns via `fit_by_priority` (Fill counts as
/// `min_cells`), then gives leftover whole cells to Fill columns (split
/// evenly, remainder to the first). `None` = dropped.
pub(crate) fn column_layout(
    columns: &[Column<'_>],
    cell_w: f32,
    width: f32,
) -> Vec<Option<ColumnSpan>> {
    if columns.is_empty() {
        return Vec::new();
    }

    let min_cells: Vec<usize> = columns
        .iter()
        .map(|c| match c.width {
            ColumnWidth::Cells(n) => n,
            ColumnWidth::Fill { min_cells } => min_cells,
        })
        .collect();
    let widths: Vec<f32> = min_cells.iter().map(|&n| n as f32 * cell_w).collect();
    let drop_priority: Vec<Option<u8>> = columns.iter().map(|c| c.drop_priority).collect();

    let gap = cell_w;
    let inset = cell_w;
    let available = (width - 2.0 * inset).max(0.0);

    let kept = fit_by_priority(&widths, &drop_priority, gap, available);

    let kept_count = kept.iter().filter(|&&k| k).count();
    let used_width: f32 = kept
        .iter()
        .zip(widths.iter())
        .filter(|&(&k, _)| k)
        .map(|(_, &w)| w)
        .sum();
    let gaps_total = if kept_count > 1 {
        gap * (kept_count - 1) as f32
    } else {
        0.0
    };
    let leftover_cells = ((available - used_width - gaps_total).max(0.0) / cell_w).floor() as usize;

    let fill_indices: Vec<usize> = (0..columns.len())
        .filter(|&i| kept[i] && matches!(columns[i].width, ColumnWidth::Fill { .. }))
        .collect();
    let mut extra_cells = vec![0usize; columns.len()];
    if !fill_indices.is_empty() {
        let base = leftover_cells / fill_indices.len();
        let remainder = leftover_cells % fill_indices.len();
        for (rank, &i) in fill_indices.iter().enumerate() {
            extra_cells[i] = base + usize::from(rank < remainder);
        }
    }

    let mut spans = vec![None; columns.len()];
    let mut x = inset;
    for i in 0..columns.len() {
        if !kept[i] {
            continue;
        }
        let cells = min_cells[i] + extra_cells[i];
        let span_width = cells as f32 * cell_w;
        spans[i] = Some(ColumnSpan {
            x,
            width: span_width,
            cells,
            align: columns[i].align,
        });
        x += span_width + gap;
    }
    spans
}

/// One row being drawn; passed to the `add_row` callback of a list view.
pub struct ListRow<'a> {
    ui: &'a mut Ui,
    rect: Rect,
    spans: &'a [Option<ColumnSpan>],
    selected: bool,
    focused: bool,
    indent: usize,
    cell_w: f32,
}

impl<'a> ListRow<'a> {
    /// Constructs one row; called by `ListView` once per visible row.
    pub(crate) fn new(
        ui: &'a mut Ui,
        rect: Rect,
        spans: &'a [Option<ColumnSpan>],
        selected: bool,
        focused: bool,
        cell_w: f32,
    ) -> Self {
        Self {
            ui,
            rect,
            spans,
            selected,
            focused,
            indent: 0,
            cell_w,
        }
    }

    /// Indents column 0 by `levels * 2` cells (for flat trees). Call before `cell(0, ..)`.
    pub fn indent(&mut self, levels: usize) {
        self.indent = levels;
    }

    /// Left edge and available width for `col`, after the row's current indent.
    fn column_geometry(&self, col: usize) -> Option<(f32, f32, usize, Align)> {
        let span = (*self.spans.get(col)?)?;
        let indent_cells = if col == 0 { self.indent * 2 } else { 0 };
        let indent_x = indent_cells as f32 * self.cell_w;
        let cells = span.cells.saturating_sub(indent_cells);
        Some((
            self.rect.min.x + span.x + indent_x,
            span.width - indent_x,
            cells,
            span.align,
        ))
    }

    /// Draws `text` in column `col`, truncated with `truncate_tail` to the
    /// column's cells and aligned per the column. `color` defaults to
    /// `selected_fg` on the selection bar and `text` elsewhere. Dropped columns are skipped.
    pub fn cell(&mut self, col: usize, text: &str, color: Option<Color32>) {
        let Some((x, width, cells, align)) = self.column_geometry(col) else {
            return;
        };

        let palette = crate::style::palette(self.ui.ctx());
        let color = color.unwrap_or(if self.selected && self.focused {
            palette.selected_fg
        } else {
            palette.text
        });

        let text = truncate_tail(text, cells);
        let font_id = TextStyle::Body.resolve(self.ui.style());
        let painter = self.ui.painter();
        let galley = painter.layout_no_wrap(text, font_id, color);

        let text_x = match align {
            Align::Min => x,
            Align::Center => x + (width - galley.size().x) / 2.0,
            Align::Max => x + width - galley.size().x,
        };
        let text_y = self.rect.center().y - galley.size().y / 2.0;
        painter.galley(pos2(text_x, text_y), galley, color);
    }

    /// Runs `add` in a child ui covering column `col`; `None` if the column is dropped.
    pub fn cell_ui<R>(&mut self, col: usize, add: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        let (x, width, _cells, _align) = self.column_geometry(col)?;
        let col_rect = Rect::from_min_size(
            pos2(x, self.rect.min.y),
            egui::vec2(width, self.rect.height()),
        );
        let mut child = self.ui.new_child(
            UiBuilder::new()
                .max_rect(col_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        child.set_clip_rect(col_rect.intersect(self.ui.clip_rect()));
        Some(add(&mut child))
    }

    /// Whether this row is the cursor row (the selection bar).
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// Full row rect (selection bar extent).
    pub fn rect(&self) -> Rect {
        self.rect
    }
}

/// Paints column titles into `rect` (the header row), truncated like cells, in `palette.dim`.
pub(crate) fn paint_header(
    painter: &Painter,
    rect: Rect,
    columns: &[Column<'_>],
    spans: &[Option<ColumnSpan>],
    palette: &Palette,
) {
    let ctx = painter.ctx();
    let font_id = TextStyle::Body.resolve(&ctx.style_of(ctx.theme()));
    for (column, span) in columns.iter().zip(spans.iter()) {
        let Some(span) = span else { continue };
        let text = truncate_tail(column.title, span.cells);
        let galley = painter.layout_no_wrap(text, font_id.clone(), palette.dim);
        let text_x = match span.align {
            Align::Min => rect.min.x + span.x,
            Align::Center => rect.min.x + span.x + (span.width - galley.size().x) / 2.0,
            Align::Max => rect.min.x + span.x + span.width - galley.size().x,
        };
        let text_y = rect.center().y - galley.size().y / 2.0;
        painter.galley(pos2(text_x, text_y), galley, palette.dim);
    }
}

/// A single-column fallback used when [`ListView::columns`] was never called:
/// one unnamed column that fills the width, and no header row.
const NO_COLUMNS: [Column<'static>; 1] = [Column {
    title: "",
    width: ColumnWidth::Fill { min_cells: 0 },
    align: Align::Min,
    drop_priority: None,
}];

/// Keyboard-driven scrolling list: column header, full-width selection bar,
/// priority column drop.
pub struct ListView<'a> {
    id_salt: Id,
    row_count: usize,
    columns: &'a [Column<'a>],
    header: bool,
}

impl<'a> ListView<'a> {
    /// New list of `row_count` rows. Header shown by default.
    // `Id::new` requires `AsId` (`Hash + Debug`) in this egui version.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug, row_count: usize) -> Self {
        Self {
            id_salt: Id::new(id_salt),
            row_count,
            columns: &[],
            header: true,
        }
    }

    /// Sets the column layout. Without this, the view uses a single unnamed
    /// fill column and shows no header.
    pub fn columns(mut self, columns: &'a [Column<'a>]) -> Self {
        self.columns = columns;
        self
    }

    /// Shows or hides the header row.
    pub fn header(mut self, show: bool) -> Self {
        self.header = show;
        self
    }

    /// Fills the available rect of `ui`. Use `response.activated` to detect
    /// Enter/double-click, not `response.response.clicked()`: Enter and Space
    /// fake a click on the outer focus target, so `clicked()` fires on every
    /// keyboard activation and select-only click alike.
    pub fn show(
        self,
        ui: &mut Ui,
        state: &mut ListState,
        mut add_row: impl FnMut(&mut ListRow<'_>, usize),
    ) -> ListResponse {
        let palette = palette(ui.ctx());
        let id = ui.make_persistent_id(self.id_salt);
        let cell_w = cell_width(ui);
        let row_h = ui.text_style_height(&TextStyle::Body);
        let row_count = self.row_count;

        let columns: &[Column<'_>] = if self.columns.is_empty() {
            &NO_COLUMNS
        } else {
            self.columns
        };
        let show_header = self.header && !self.columns.is_empty();

        let start_cursor = state.cursor;
        state.cursor = if row_count == 0 {
            0
        } else {
            state.cursor.min(row_count - 1)
        };

        let outer_rect = ui.available_rect_before_wrap();
        // Interacted first so row widgets, added later, take hit-test
        // priority over this catch-all "click on empty space focuses" area.
        let outer_response = ui.interact(outer_rect, id, Sense::click());
        if outer_response.clicked() {
            ui.ctx().memory_mut(|m| m.request_focus(id));
        }
        let focused = outer_response.has_focus();
        if focused {
            ui.ctx().memory_mut(|m| {
                m.set_focus_lock_filter(
                    id,
                    EventFilter {
                        vertical_arrows: true,
                        ..Default::default()
                    },
                );
            });
        }

        // Scoped so the 0 row spacing doesn't leak into the caller's `ui`;
        // the ScrollArea content and header allocation below still inherit
        // it since they're nested inside this scope.
        let activated = ui
            .scope(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;

                let header_rect = if show_header {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(outer_rect.width(), row_h), Sense::hover());
                    rect
                } else {
                    Rect::NOTHING
                };

                let mut activated = None;
                let mut header_spans: Vec<Option<ColumnSpan>> = Vec::new();

                ScrollArea::vertical()
                    .id_salt(id.with("scroll"))
                    .auto_shrink([false, false])
                    .content_margin(0.0)
                    .show_viewport(ui, |ui, viewport| {
                        let content_width = ui.max_rect().width();
                        let top_left = ui.max_rect().min;
                        let spans = column_layout(columns, cell_w, content_width);
                        header_spans = spans.clone();

                        ui.set_height(row_count as f32 * row_h);

                        let mut cursor_moved_by_keys = false;
                        if focused && row_count > 0 {
                            let visible_rows =
                                (viewport.height() / row_h).floor().max(1.0) as usize;
                            let page = visible_rows.saturating_sub(1).max(1);
                            let prev_cursor = state.cursor;
                            ui.ctx().input_mut(|input| {
                                let down =
                                    input.count_and_consume_key(Modifiers::NONE, Key::ArrowDown);
                                state.cursor = state.cursor.saturating_add(down).min(row_count - 1);

                                let up = input.count_and_consume_key(Modifiers::NONE, Key::ArrowUp);
                                state.cursor = state.cursor.saturating_sub(up);

                                let page_down =
                                    input.count_and_consume_key(Modifiers::NONE, Key::PageDown);
                                state.cursor = state
                                    .cursor
                                    .saturating_add(page_down.saturating_mul(page))
                                    .min(row_count - 1);

                                let page_up =
                                    input.count_and_consume_key(Modifiers::NONE, Key::PageUp);
                                state.cursor =
                                    state.cursor.saturating_sub(page_up.saturating_mul(page));

                                if input.count_and_consume_key(Modifiers::NONE, Key::Home) > 0 {
                                    state.cursor = 0;
                                }
                                if input.count_and_consume_key(Modifiers::NONE, Key::End) > 0 {
                                    state.cursor = row_count - 1;
                                }
                                if input.count_and_consume_key(Modifiers::NONE, Key::Enter) > 0 {
                                    activated = Some(state.cursor);
                                }
                            });
                            cursor_moved_by_keys = state.cursor != prev_cursor;
                        }

                        if cursor_moved_by_keys {
                            let cursor_rect = Rect::from_min_size(
                                top_left + vec2(0.0, state.cursor as f32 * row_h),
                                vec2(content_width, row_h),
                            );
                            ui.scroll_to_rect_animation(cursor_rect, None, ScrollAnimation::none());
                        }

                        let min_row = (viewport.min.y / row_h).floor().max(0.0) as usize;
                        let max_row =
                            ((viewport.max.y / row_h).ceil().max(0.0) as usize).min(row_count);

                        for row in min_row..max_row {
                            let row_rect = Rect::from_min_size(
                                top_left + vec2(0.0, row as f32 * row_h),
                                vec2(content_width, row_h),
                            );

                            let row_response =
                                ui.interact(row_rect, id.with(("row", row)), Sense::CLICK);
                            if row_response.clicked() {
                                state.cursor = row;
                                ui.ctx().memory_mut(|m| m.request_focus(id));
                            }
                            if row_response.double_clicked() {
                                activated = Some(row);
                            }

                            let selected = row == state.cursor;
                            if selected {
                                if focused {
                                    ui.painter().rect_filled(row_rect, 0, palette.selected_bg);
                                } else {
                                    paint_frame(ui.painter(), row_rect, palette.selected_bg);
                                }
                            }

                            let mut list_row =
                                ListRow::new(ui, row_rect, &spans, selected, focused, cell_w);
                            add_row(&mut list_row, row);
                        }
                    });

                if show_header {
                    paint_header(ui.painter(), header_rect, columns, &header_spans, &palette);
                }

                activated
            })
            .inner;

        ListResponse {
            response: outer_response,
            activated,
            cursor_changed: state.cursor != start_cursor,
        }
    }
}

/// Result of [`ListView::show`].
pub struct ListResponse {
    /// Focus target of the list; `response.has_focus()` tells whether keys go
    /// to it. Do not use `response.clicked()` to detect activation: Enter and
    /// Space fake a click on this response when the list has focus, so it
    /// fires on every keyboard activation, not just a mouse click. Use
    /// `activated` instead.
    pub response: Response,
    /// Row activated this frame by Enter or double-click.
    pub activated: Option<usize>,
    /// Whether `state.cursor` differs from its value at the start of `show`.
    pub cursor_changed: bool,
}

#[cfg(test)]
mod tests {
    use egui::{Event, RawInput};

    use crate::Palette;
    use crate::style::apply_with;

    use super::*;

    #[test]
    fn fixed_columns_positions() {
        let columns = [
            Column::new("A", ColumnWidth::Cells(4)),
            Column::new("B", ColumnWidth::Cells(4)),
        ];
        let spans = column_layout(&columns, 8.0, 200.0);
        let a = spans[0].unwrap();
        let b = spans[1].unwrap();
        assert_eq!(a.x, 8.0);
        assert_eq!(a.width, 32.0);
        assert_eq!(b.x, 48.0);
        assert_eq!(b.width, 32.0);
    }

    #[test]
    fn fill_takes_leftover() {
        let columns = [
            Column::new("A", ColumnWidth::Cells(4)),
            Column::new("B", ColumnWidth::Fill { min_cells: 2 }),
        ];
        let spans = column_layout(&columns, 8.0, 200.0);
        let a = spans[0].unwrap();
        let b = spans[1].unwrap();
        assert_eq!(a.x, 8.0);
        assert_eq!(a.width, 32.0);
        // available = 200 - 16 = 184; used = 32 + 16 (min) + 8 (gap) = 56;
        // leftover = 128 pts = 16 cells, all going to the sole Fill column.
        assert_eq!(b.cells, 18);
        assert_eq!(b.width, 144.0);
        assert_eq!(b.x, 48.0);
    }

    #[test]
    fn drops_by_priority_when_narrow() {
        // B (middle) has the higher drop priority, C (rightmost) the lower
        // one, so a position-based (rightmost-first) implementation would
        // drop C and fail this assertion; only priority should decide here.
        let columns = [
            Column::new("A", ColumnWidth::Cells(5)),
            Column::new("B", ColumnWidth::Cells(5)).drop_priority(2),
            Column::new("C", ColumnWidth::Cells(5)).drop_priority(1),
        ];
        // available = 136 - 16 = 120; all three need 40*3 + 8*2 = 136 > 120,
        // so the highest-priority column (B) is dropped; A + C = 88 fits.
        let spans = column_layout(&columns, 8.0, 136.0);
        assert!(spans[0].is_some());
        assert!(spans[1].is_none());
        assert!(spans[2].is_some());
    }

    #[test]
    fn never_drops_none_columns() {
        let columns = [
            Column::new("A", ColumnWidth::Cells(5)),
            Column::new("B", ColumnWidth::Cells(5)),
        ];
        // width leaves no room past the insets; both columns have no drop
        // priority, so `fit_by_priority` must keep them regardless.
        let spans = column_layout(&columns, 8.0, 16.0);
        assert!(spans[0].is_some());
        assert!(spans[1].is_some());
    }

    #[test]
    fn list_row_dropped_column_is_skipped() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let columns = [
            Column::new("A", ColumnWidth::Cells(4)),
            Column::new("B", ColumnWidth::Cells(4)).drop_priority(1),
        ];
        // Narrow enough that B is dropped.
        let spans = column_layout(&columns, 8.0, 40.0);
        assert!(spans[1].is_none());

        let mut cell_ui_result = Some(());
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let rect = ui.available_rect_before_wrap();
            let mut row = ListRow::new(ui, rect, &spans, false, true, 8.0);
            row.cell(1, "should be skipped", None);
            cell_ui_result = row.cell_ui(1, |_ui| ());
        });
        output.textures_delta.clear();

        assert!(cell_ui_result.is_none());
    }

    #[test]
    fn paint_header_smoke_test() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let columns = [
            Column::new("NAME", ColumnWidth::Fill { min_cells: 4 }),
            Column::new("SIZE", ColumnWidth::Cells(4)).align(Align::Max),
        ];
        let spans = column_layout(&columns, 8.0, 200.0);

        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let rect = ui.available_rect_before_wrap();
            paint_header(ui.painter(), rect, &columns, &spans, &Palette::default());
        });
        output.textures_delta.clear();
    }

    /// Draws one `ListView` over `row_count` rows in a fixed-size area,
    /// followed by a focusable button. The button lets tests check that
    /// keyboard navigation (e.g. Tab) doesn't steal focus away from the
    /// list onto a row, since rows must not be individually focusable.
    fn show_list(
        ctx: &egui::Context,
        id_salt: &str,
        row_count: usize,
        state: &mut ListState,
        events: Vec<Event>,
    ) -> ListResponse {
        let mut response = None;
        let mut output = ctx.run_ui(
            RawInput {
                events,
                ..Default::default()
            },
            |ui| {
                response = Some(
                    ui.allocate_ui(vec2(300.0, 200.0), |ui| {
                        ListView::new(id_salt, row_count).show(ui, state, |row, i| {
                            row.cell(0, &format!("Row {i}"), None);
                        })
                    })
                    .inner,
                );
                let _ = ui.button("below the list");
            },
        );
        output.textures_delta.clear();
        response.unwrap()
    }

    #[test]
    fn list_view_no_input_smoke() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut state = ListState::default();
        let response = show_list(&ctx, "list_smoke", 10, &mut state, Vec::new());

        assert_eq!(state.cursor, 0);
        assert_eq!(response.activated, None);
    }

    #[test]
    fn list_view_clamps_cursor() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut state = ListState { cursor: 99 };
        let response = show_list(&ctx, "list_clamp", 10, &mut state, Vec::new());

        assert_eq!(state.cursor, 9);
        assert!(response.cursor_changed);
    }

    #[test]
    fn list_view_arrow_down_moves_cursor_when_focused() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut state = ListState::default();

        // Frame 1: register the widget and grab its id.
        let id = show_list(&ctx, "list_arrow", 10, &mut state, Vec::new())
            .response
            .id;
        ctx.memory_mut(|m| m.request_focus(id));

        // Frame 2: re-register with focus now set, no input yet.
        // `Memory::set_focus_lock_filter` only takes effect if the widget
        // already had focus *last* frame, so this warm-up frame is needed
        // before arrow keys are routed to the list instead of being treated
        // as a focus-change request.
        show_list(&ctx, "list_arrow", 10, &mut state, Vec::new());

        // Frame 3: send ArrowDown.
        let response = show_list(
            &ctx,
            "list_arrow",
            10,
            &mut state,
            vec![Event::Key {
                key: Key::ArrowDown,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );

        assert_eq!(state.cursor, 1);
        assert!(response.cursor_changed);
        // The button placed after the list in `show_list` must not have
        // stolen focus: rows are not focusable, so only the list itself
        // handles the vertical-arrows event filter.
        assert!(ctx.memory(|m| m.has_focus(id)));
    }

    #[test]
    fn list_view_end_and_enter() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut state = ListState::default();
        let row_count = 10;

        let id = show_list(&ctx, "list_end_enter", row_count, &mut state, Vec::new())
            .response
            .id;
        ctx.memory_mut(|m| m.request_focus(id));
        show_list(&ctx, "list_end_enter", row_count, &mut state, Vec::new());

        show_list(
            &ctx,
            "list_end_enter",
            row_count,
            &mut state,
            vec![Event::Key {
                key: Key::End,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
        assert_eq!(state.cursor, row_count - 1);

        // No extra warm-up frame needed here: the list kept focus across the
        // End frame, so `set_focus_lock_filter`'s had-focus-last-frame check
        // is already satisfied for this Enter frame.
        let response = show_list(
            &ctx,
            "list_end_enter",
            row_count,
            &mut state,
            vec![Event::Key {
                key: Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );

        assert_eq!(response.activated, Some(row_count - 1));
    }

    #[test]
    fn list_view_keys_ignored_without_focus() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut state = ListState::default();

        // Two warm-up frames, no focus requested.
        show_list(&ctx, "list_unfocused", 10, &mut state, Vec::new());
        show_list(&ctx, "list_unfocused", 10, &mut state, Vec::new());

        let response = show_list(
            &ctx,
            "list_unfocused",
            10,
            &mut state,
            vec![Event::Key {
                key: Key::ArrowDown,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );

        assert_eq!(state.cursor, 0);
        assert!(!response.cursor_changed);
    }

    #[test]
    fn list_view_does_not_leak_item_spacing_into_caller() {
        let ctx = egui::Context::default();
        apply_with(&ctx, &Palette::default());

        let mut state = ListState::default();
        let mut spacing_before_and_after: Option<(f32, f32)> = None;
        let mut output = ctx.run_ui(RawInput::default(), |ui| {
            let before = ui.spacing().item_spacing.y;
            assert_ne!(before, 0.0, "test setup: default spacing must be nonzero");

            ListView::new("list_spacing", 10).show(ui, &mut state, |row, i| {
                row.cell(0, &format!("Row {i}"), None);
            });

            spacing_before_and_after = Some((before, ui.spacing().item_spacing.y));
        });
        output.textures_delta.clear();

        let (before, after) = spacing_before_and_after.unwrap();
        assert_eq!(before, after);
    }
}
