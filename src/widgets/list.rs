//! List column layout and the per-row drawing primitive: fixed/fill columns
//! with priority-based dropping, and cell drawing truncated to fit.

use egui::{Align, Color32, Layout, Painter, Rect, TextStyle, Ui, UiBuilder, pos2};

use crate::Palette;
use crate::layout::fit_by_priority;
use crate::text::truncate_tail;

/// Width policy of a [`Column`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColumnWidth {
    /// Fixed width in half-width cells.
    Cells(usize),
    /// Takes the width left over; never narrower than `min_cells`.
    Fill { min_cells: usize },
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
// Not yet called outside tests: `ListView` (a later change) will use it to
// lay out both the header row and each body row.
#[cfg_attr(not(test), allow(dead_code))]
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
    // Not yet called outside tests: `ListView` (a later change) constructs
    // one `ListRow` per visible row from here.
    #[cfg_attr(not(test), allow(dead_code))]
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
        child.set_clip_rect(col_rect);
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
// Not yet called outside tests: `ListView` (a later change) paints the header row with it.
#[cfg_attr(not(test), allow(dead_code))]
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

#[cfg(test)]
mod tests {
    use egui::RawInput;

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
}
