# ANSI Art

Ubah gambar (PNG, JPG, WebP, GIF, …) jadi ANSI art untuk terminal — seperti logo distro di neofetch/fastfetch.
Satu engine Rust, dipakai oleh app desktop (Tauri) dan CLI.

```
ansi-art/
├── crates/
│   ├── ansi-core/          Engine: load → resample → adjust → dither → render → export
│   │   └── src/
│   │       ├── render/     Mode render (glyph, ascii, shade, block, half-block, quadrant, sextant, braille)
│   │       ├── export/     Format output (ansi, plain, html, json, sh, ps1, neofetch, fastfetch)
│   │       ├── color.rs    Palet & kuantisasi (truecolor / 256 / 16 / 8 / mono)
│   │       ├── dither.rs   Floyd–Steinberg, Atkinson, Bayer 4×4 / 8×8
│   │       └── pixels.rs   Resample (alpha premultiplied) + brightness/contrast/gamma/…
│   └── assets/             Font JetBrains Mono (OFL) untuk glyph matching
│   └── ansi-cli/           CLI `ansi-art`
├── app/                    App desktop: Tauri 2 + Svelte 5 + xterm.js
│   ├── src/                Frontend (UI + preview terminal)
│   └── src-tauri/          Backend Tauri (command load/render/export)
└── samples/                Gambar uji
```

## Prasyarat

- Rust (stable) — https://rustup.rs
- Node.js 20+ dan pnpm
- Dependensi sistem Tauri: https://v2.tauri.app/start/prerequisites/
  - **Windows**: WebView2 (sudah ada di Windows 10/11) + MSVC Build Tools
  - **Linux**: `webkit2gtk-4.1`, `libayatana-appindicator3`, `librsvg2` (lihat link di atas per distro)
  - **macOS**: Xcode Command Line Tools

## Menjalankan

```sh
# App desktop (hot reload)
cd app
pnpm install
pnpm tauri dev

# Build installer (.msi/.exe, .deb/.rpm/.AppImage, .dmg)
pnpm tauri build

# CLI
cargo run -p ansi-cli --release -- samples/logo.png -m sextant -w 50
cargo install --path crates/ansi-cli   # pasang `ansi-art` ke PATH
```

### Contoh CLI

```sh
ansi-art logo.png                                   # half-block, warna terdeteksi otomatis
ansi-art logo.png -m braille -c ansi256 -d bayer8   # braille, 256 warna, ordered dither
ansi-art logo.png -m ascii --ramp detailed -c mono  # ASCII klasik
ansi-art logo.png -m quadrant -o logo.ans           # format diambil dari ekstensi
ansi-art logo.png -o logo.ps1                       # skrip PowerShell yang mencetak logo
ansi-art --help                                     # semua opsi
```

### Mode Glyph (gaya logo neofetch)

Karakter dipilih berdasarkan **bentuk**: tepi bawah jadi `.` `_`, tepi atas `'` `` ` ``, sisi `|` `:`,
area penuh diisi karakter pilihan (`M`, `c`, `#`, …). Warna direduksi ke beberapa warna flat (k-means).
Latar belakang dideteksi dari transparansi, atau dari warna tepi gambar untuk JPG.

```sh
ansi-art logo.png -m glyph -w 40                               # default: charset edges, fill M, 3 warna
ansi-art logo.png -m glyph --charset letters --fill c          # tepi dari huruf, isi 'c'
ansi-art logo.png -m glyph --glyph-colors 2 -f neofetch -o logo.txt
#   → mencetak: neofetch --ascii logo.txt --ascii_colors 40 21
ansi-art logo.png -m glyph -f fastfetch -o logo.txt
#   → mencetak: fastfetch --logo logo.txt --logo-type file --logo-color-1 '38;2;…' …
```

Format `neofetch`/`fastfetch` hanya menyimpan warna foreground (maks. 6 / 9 warna),
jadi paling cocok untuk mode glyph, ascii, dan braille.

### Pakai sebagai logo fastfetch / MOTD

```sh
ansi-art logo.png -w 40 -o ~/.config/fastfetch/logo.ans
fastfetch --file-raw ~/.config/fastfetch/logo.ans
ansi-art logo.png -w 60 -c ansi256 | sudo tee /etc/motd
```

## Kompatibilitas

| Hal | Cara ditangani |
|---|---|
| Kedalaman warna | Truecolor → 256 → 16 → 8 → mono. CLI mendeteksi dari `COLORTERM`, `TERM`, `WT_SESSION`, menghormati `NO_COLOR`. |
| Warna 256 | Hanya memakai indeks 16–255 (tetap di semua terminal); indeks 0–15 berubah per tema. |
| Console Windows lama | CLI mengaktifkan VT processing otomatis. Skrip `.ps1` disimpan dengan BOM agar PowerShell 5.1 membacanya sebagai UTF-8. |
| Font | Half-block/quadrant/braille ada di hampir semua font. Sextant butuh font baru (Cascadia, JetBrains Mono, Iosevka). Kalau ragu: `block` atau `ascii`. |
| Transparansi | Pixel transparan jadi sel kosong (background terminal tetap terlihat). |
| Baris | Setiap baris diakhiri reset `ESC[0m` — aman dipotong/digabung. |

### Catatan WebKitGTK (Linux)

Semua pemrosesan gambar ada di Rust; webview hanya menerima string ANSI. Preview memakai xterm.js
dengan renderer WebGL dan **otomatis turun ke DOM** kalau WebGL gagal atau context hilang
(bisa dimatikan manual lewat toggle "WebGL"). Pada GPU NVIDIA, app menyetel
`WEBKIT_DISABLE_DMABUF_RENDERER=1` (kecuali sudah Anda set sendiri) untuk menghindari jendela kosong.

## Roadmap

- Render: ASCII edge (Sobel), octant (Unicode 16), blue-noise dither
- Export: PNG & SVG (glyph blok digambar sebagai geometri),
  `.ans` CP437 + SAUCE, snippet kode (C/Rust/Python/Go), mIRC
- Kuantisasi warna di ruang warna perseptual (OKLab), palet custom
- Opsi crop otomatis, background color, resize filter
- Dukungan GIF animasi
