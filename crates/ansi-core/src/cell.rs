use serde::Serialize;

use crate::color::Rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Cell {
    pub ch: char,
    pub fg: Option<Rgb>,
    pub bg: Option<Rgb>,
}

impl Cell {
    pub const EMPTY: Cell = Cell { ch: ' ', fg: None, bg: None };

    /// True when the cell shows nothing on the terminal's default background.
    pub fn is_blank(&self) -> bool {
        self.ch == ' ' && self.bg.is_none()
    }
}

/// Renderer output: true-color cells. Exporters quantize to the target color mode.
#[derive(Clone, Debug, Serialize)]
pub struct Grid {
    pub cols: usize,
    pub rows: usize,
    pub cells: Vec<Cell>,
}

impl Grid {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self { cols, rows, cells: vec![Cell::EMPTY; cols * rows] }
    }

    pub fn row(&self, y: usize) -> &[Cell] {
        &self.cells[y * self.cols..(y + 1) * self.cols]
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        self.cells[y * self.cols + x] = cell;
    }

    /// Row without trailing blank cells.
    pub fn trimmed_row(&self, y: usize) -> &[Cell] {
        let row = self.row(y);
        let end = row.iter().rposition(|c| !c.is_blank()).map_or(0, |i| i + 1);
        &row[..end]
    }
}
