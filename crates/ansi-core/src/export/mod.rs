//! Output formats. Every exporter takes the true-color grid and quantizes to
//! the requested color mode itself, so one render can be saved many ways.

pub mod ansi;
pub mod html;
pub mod logo;
pub mod script;

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{cell::Grid, color::ColorMode, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    /// Raw ANSI escape codes (UTF-8); `cat` it, use it as MOTD or fastfetch `--file-raw`
    Ansi,
    /// Characters only, no color
    Plain,
    /// Standalone HTML page
    Html,
    /// Cell grid as JSON, for other tools
    Json,
    /// POSIX shell script that prints the art
    Sh,
    /// PowerShell script that prints the art
    Ps1,
    /// neofetch custom logo with `${c1}`..`${c6}` color placeholders
    Neofetch,
    /// fastfetch custom logo with `$1`..`$9` color placeholders
    Fastfetch,
}

impl Format {
    pub const ALL: [Format; 8] = [
        Format::Ansi,
        Format::Plain,
        Format::Html,
        Format::Json,
        Format::Sh,
        Format::Ps1,
        Format::Neofetch,
        Format::Fastfetch,
    ];

    pub fn extension(self) -> &'static str {
        match self {
            Format::Ansi => "ans",
            Format::Plain | Format::Neofetch | Format::Fastfetch => "txt",
            Format::Html => "html",
            Format::Json => "json",
            Format::Sh => "sh",
            Format::Ps1 => "ps1",
        }
    }

    /// Guesses from the extension; `.txt` means plain (logo formats must be asked for).
    pub fn from_path(path: &Path) -> Option<Format> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        Format::ALL.into_iter().find(|f| f.extension() == ext).or(match ext.as_str() {
            "ansi" => Some(Format::Ansi),
            "htm" => Some(Format::Html),
            _ => None,
        })
    }
}

pub struct Output {
    pub text: String,
    /// How to use the file, with `{file}` standing in for its path (e.g. the neofetch command).
    pub hint: Option<String>,
}

impl Output {
    /// The hint with the real file path filled in (quoted when it has spaces).
    pub fn hint_for(&self, file: &str) -> Option<String> {
        let file = if file.contains(' ') { format!("\"{file}\"") } else { file.to_string() };
        self.hint.as_ref().map(|h| h.replace("{file}", &file))
    }
}

impl From<String> for Output {
    fn from(text: String) -> Self {
        Output { text, hint: None }
    }
}

pub fn export(grid: &Grid, format: Format, color: ColorMode) -> Result<Output> {
    Ok(match format {
        Format::Ansi => ansi::encode(grid, color).into(),
        Format::Plain => plain(grid).into(),
        Format::Html => html::encode(grid, color).into(),
        Format::Json => serde_json::to_string(grid)?.into(),
        Format::Sh => script::sh(&ansi::encode(grid, color)).into(),
        Format::Ps1 => script::ps1(&ansi::encode(grid, color)).into(),
        Format::Neofetch => logo::neofetch(grid, color),
        Format::Fastfetch => logo::fastfetch(grid, color),
    })
}

fn plain(grid: &Grid) -> String {
    let mut out = String::with_capacity(grid.cols * grid.rows + grid.rows);
    for y in 0..grid.rows {
        out.extend(grid.trimmed_row(y).iter().map(|c| c.ch));
        out.push('\n');
    }
    out
}
