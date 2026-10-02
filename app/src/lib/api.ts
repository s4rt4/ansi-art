// Typed bridge to the Rust commands in src-tauri/src/lib.rs.
// Shapes mirror ansi-core's serde types (camelCase fields, kebab-case enums).

import { invoke } from "@tauri-apps/api/core";

export type Mode = "ascii" | "shade" | "block" | "half-block" | "quadrant" | "sextant" | "braille" | "glyph";
export type ColorMode = "truecolor" | "ansi256" | "ansi16" | "ansi8" | "mono";
export type Dither = "none" | "floyd-steinberg" | "atkinson" | "bayer4" | "bayer8";
export type Format = "ansi" | "plain" | "html" | "json" | "sh" | "ps1" | "neofetch" | "fastfetch";

export interface Adjust {
  brightness: number;
  contrast: number;
  gamma: number;
  saturation: number;
  invert: boolean;
}

export interface GlyphOptions {
  charset: string;
  fill: string;
  fillLevel: number;
  colors: number;
}

export interface RenderOptions {
  mode: Mode;
  width: number;
  height: number | null;
  cellAspect: number;
  color: ColorMode;
  dither: Dither;
  ramp: string;
  threshold: number;
  alphaThreshold: number;
  adjust: Adjust;
  glyph: GlyphOptions;
}

export interface ImageInfo {
  width: number;
  height: number;
}

export interface Preview {
  ansi: string;
  cols: number;
  rows: number;
}

/** Same values as `RenderOptions::default()` in Rust. */
export const defaultOptions = (): RenderOptions => ({
  mode: "half-block",
  width: 60,
  height: null,
  cellAspect: 0.5,
  color: "truecolor",
  dither: "none",
  ramp: "standard",
  threshold: 0.5,
  alphaThreshold: 0.5,
  adjust: { brightness: 0, contrast: 1, gamma: 1, saturation: 1, invert: false },
  glyph: { charset: "edges", fill: "M", fillLevel: 0.8, colors: 3 },
});

export const MODES: { value: Mode; label: string; hint: string }[] = [
  { value: "glyph", label: "Glyph (logo)", hint: "Shape-matched ASCII, neofetch-logo style" },
  { value: "half-block", label: "Half block ▀", hint: "1×2 px/cell, fastfetch-style" },
  { value: "quadrant", label: "Quadrant ▚", hint: "2×2 px/cell, two colors" },
  { value: "sextant", label: "Sextant 🬗", hint: "2×3 px/cell, needs a recent font" },
  { value: "braille", label: "Braille ⣿", hint: "2×4 dots, one color" },
  { value: "block", label: "Full block █", hint: "1 px/cell, safest" },
  { value: "shade", label: "Shade ░▒▓", hint: "Brightness via shading" },
  { value: "ascii", label: "ASCII", hint: "Character ramp" },
];

export const COLORS: { value: ColorMode; label: string }[] = [
  { value: "truecolor", label: "Truecolor (24-bit)" },
  { value: "ansi256", label: "256 colors" },
  { value: "ansi16", label: "16 colors" },
  { value: "ansi8", label: "8 colors" },
  { value: "mono", label: "Monochrome" },
];

export const DITHERS: { value: Dither; label: string }[] = [
  { value: "none", label: "None" },
  { value: "floyd-steinberg", label: "Floyd–Steinberg" },
  { value: "atkinson", label: "Atkinson" },
  { value: "bayer4", label: "Bayer 4×4" },
  { value: "bayer8", label: "Bayer 8×8" },
];

/** Mirrors `render::glyph::CHARSETS`. */
export const CHARSETS = ["edges", "letters", "all", "blocks"];

/** Mirrors `render::ramps::PRESETS`. */
export const RAMPS = ["standard", "detailed", "simple", "blocks", "dots", "lines", "binary"];

export const FORMATS: { value: Format; label: string; ext: string }[] = [
  { value: "ansi", label: "ANSI (.ans)", ext: "ans" },
  { value: "plain", label: "Plain text (.txt)", ext: "txt" },
  { value: "html", label: "HTML page (.html)", ext: "html" },
  { value: "json", label: "JSON cells (.json)", ext: "json" },
  { value: "sh", label: "Shell script (.sh)", ext: "sh" },
  { value: "ps1", label: "PowerShell (.ps1)", ext: "ps1" },
  { value: "neofetch", label: "neofetch logo (.txt)", ext: "txt" },
  { value: "fastfetch", label: "fastfetch logo (.txt)", ext: "txt" },
];

export const loadImage = (path: string) => invoke<ImageInfo>("load_image", { path });

export const renderPreview = (options: RenderOptions) => invoke<Preview>("render_preview", { options });

/** Resolves to a usage hint (e.g. the neofetch command line) for formats that have one. */
export const exportFile = (options: RenderOptions, format: Format, path: string) =>
  invoke<string | null>("export_file", { options, format, path });
