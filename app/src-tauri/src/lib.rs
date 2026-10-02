//! Tauri shell around ansi-core.
//!
//! Only paths, options and finished text cross the IPC bridge; images are decoded
//! and processed here, never shipped to the webview (slow on WebKitGTK).

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use ansi_core::{export, DynamicImage, Format, RenderOptions};
use serde::Serialize;
use tauri::{async_runtime::spawn_blocking, State};

#[derive(Default)]
struct AppState {
    image: Mutex<Option<Arc<DynamicImage>>>,
}

impl AppState {
    fn current(&self) -> Result<Arc<DynamicImage>, String> {
        self.image.lock().unwrap().clone().ok_or_else(|| "no image loaded".into())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageInfo {
    width: u32,
    height: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Preview {
    ansi: String,
    cols: usize,
    rows: usize,
}

/// Runs CPU-heavy work off the async runtime and flattens both error layers.
async fn blocking<T: Send + 'static>(f: impl FnOnce() -> ansi_core::Result<T> + Send + 'static) -> Result<T, String> {
    spawn_blocking(f).await.map_err(|e| e.to_string())?.map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_image(path: PathBuf, state: State<'_, AppState>) -> Result<ImageInfo, String> {
    let img = blocking(move || ansi_core::load(path)).await?;
    let info = ImageInfo { width: img.width(), height: img.height() };
    *state.image.lock().unwrap() = Some(Arc::new(img));
    Ok(info)
}

#[tauri::command]
async fn render_preview(options: RenderOptions, state: State<'_, AppState>) -> Result<Preview, String> {
    let img = state.current()?;
    blocking(move || {
        let grid = ansi_core::convert(&img, &options);
        let ansi = export::ansi::encode(&grid, options.color);
        Ok(Preview { ansi, cols: grid.cols, rows: grid.rows })
    })
    .await
}

#[tauri::command]
async fn export_file(
    options: RenderOptions,
    format: Format,
    path: PathBuf,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let img = state.current()?;
    blocking(move || {
        let grid = ansi_core::convert(&img, &options);
        let out = export::export(&grid, format, options.color)?;
        std::fs::write(&path, &out.text)?;
        Ok(out.hint_for(&path.display().to_string()))
    })
    .await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    linux_webkit_workarounds();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![load_image, render_preview, export_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// WebKitGTK's DMA-BUF renderer shows a blank window on many NVIDIA setups.
/// Disable it there unless the user already decided via the env var.
fn linux_webkit_workarounds() {
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none()
        && std::path::Path::new("/proc/driver/nvidia/version").exists()
    {
        // Runs first thing in `run`, before any other thread exists.
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
}
