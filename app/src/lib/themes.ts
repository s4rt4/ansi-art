// Terminal palettes for checking how 16/8-color output looks elsewhere.
// Truecolor and 256-color (16..255) output is unaffected by these.

import type { ITheme } from "@xterm/xterm";

const palette = (
  background: string,
  foreground: string,
  [black, red, green, yellow, blue, magenta, cyan, white,
   brightBlack, brightRed, brightGreen, brightYellow, brightBlue, brightMagenta, brightCyan, brightWhite]: string[],
): ITheme => ({
  background, foreground, cursor: background,
  black, red, green, yellow, blue, magenta, cyan, white,
  brightBlack, brightRed, brightGreen, brightYellow, brightBlue, brightMagenta, brightCyan, brightWhite,
});

const XTERM = [
  "#000000", "#cd0000", "#00cd00", "#cdcd00", "#0000ee", "#cd00cd", "#00cdcd", "#e5e5e5",
  "#7f7f7f", "#ff0000", "#00ff00", "#ffff00", "#5c5cff", "#ff00ff", "#00ffff", "#ffffff",
];

export const THEMES: Record<string, ITheme> = {
  "xterm (reference)": palette("#000000", "#e5e5e5", XTERM),
  "Windows Terminal (Campbell)": palette("#0c0c0c", "#cccccc", [
    "#0c0c0c", "#c50f1f", "#13a10e", "#c19c00", "#0037da", "#881798", "#3a96dd", "#cccccc",
    "#767676", "#e74856", "#16c60c", "#f9f1a5", "#3b78ff", "#b4009e", "#61d6d6", "#f2f2f2",
  ]),
  "Ubuntu (Tango)": palette("#300a24", "#ffffff", [
    "#2e3436", "#cc0000", "#4e9a06", "#c4a000", "#3465a4", "#75507b", "#06989a", "#d3d7cf",
    "#555753", "#ef2929", "#8ae234", "#fce94f", "#729fcf", "#ad7fa8", "#34e2e2", "#eeeeec",
  ]),
  "Solarized Dark": palette("#002b36", "#839496", [
    "#073642", "#dc322f", "#859900", "#b58900", "#268bd2", "#d33682", "#2aa198", "#eee8d5",
    "#002b36", "#cb4b16", "#586e75", "#657b83", "#839496", "#6c71c4", "#93a1a1", "#fdf6e3",
  ]),
  "Light background": palette("#ffffff", "#000000", XTERM),
};

export const DEFAULT_THEME = "Windows Terminal (Campbell)";
