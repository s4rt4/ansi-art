<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import {
    CHARSETS, COLORS, DITHERS, FORMATS, MODES, RAMPS,
    defaultOptions, exportFile, loadImage, renderPreview,
    type Format, type ImageInfo, type Preview,
  } from "./lib/api";
  import { DEFAULT_THEME, THEMES } from "./lib/themes";
  import Slider from "./lib/Slider.svelte";
  import Terminal, { type Renderer } from "./lib/Terminal.svelte";

  const IMAGE_EXTENSIONS = ["png", "jpg", "jpeg", "webp", "gif", "bmp", "tif", "tiff", "ico", "tga", "qoi"];

  let opts = $state(defaultOptions());
  let image = $state<(ImageInfo & { path: string; name: string }) | null>(null);
  let preview = $state<Preview>({ ansi: "", cols: 60, rows: 20 });
  let busy = $state(false);
  let message = $state<{ kind: "error" | "info"; text: string; code?: string } | null>(null);
  let dragging = $state(false);

  let themeName = $state(DEFAULT_THEME);
  let fontSize = $state(14);
  let preferWebgl = $state(true);
  let renderer = $state<Renderer>("dom");
  let exportFormat = $state<Format>("ansi");

  const isPreset = $derived(RAMPS.includes(opts.ramp));
  const isGlyph = $derived(opts.mode === "glyph");
  const isCharset = $derived(CHARSETS.includes(opts.glyph.charset));
  const usesRamp = $derived(opts.mode === "ascii");
  const usesThreshold = $derived(opts.mode === "braille" || opts.color === "mono");

  const basename = (path: string) => path.split(/[\\/]/).pop() ?? path;

  async function openPath(path: string) {
    try {
      const info = await loadImage(path);
      image = { ...info, path, name: basename(path) };
      message = null;
    } catch (err) {
      message = { kind: "error", text: `Could not open ${basename(path)}: ${err}` };
    }
  }

  async function pickFile() {
    const path = await open({ multiple: false, filters: [{ name: "Images", extensions: IMAGE_EXTENSIONS }] });
    if (typeof path === "string") await openPath(path);
  }

  async function doExport() {
    if (!image) return;
    const format = FORMATS.find((f) => f.value === exportFormat)!;
    const stem = image.name.replace(/\.[^.]+$/, "");
    const path = await save({
      defaultPath: `${stem}.${format.ext}`,
      filters: [{ name: format.label, extensions: [format.ext] }],
    });
    if (!path) return;
    try {
      const hint = await exportFile($state.snapshot(opts), exportFormat, path);
      message = { kind: "info", text: `Saved ${path}`, code: hint ?? undefined };
    } catch (err) {
      message = { kind: "error", text: `Export failed: ${err}` };
    }
  }

  // Debounced re-render; `seq` drops responses that a newer request has superseded.
  let seq = 0;
  $effect(() => {
    const snapshot = $state.snapshot(opts);
    if (!image) return;
    const id = ++seq;
    const timer = setTimeout(async () => {
      busy = true;
      try {
        const result = await renderPreview(snapshot);
        if (id === seq) preview = result;
      } catch (err) {
        if (id === seq) message = { kind: "error", text: String(err) };
      } finally {
        if (id === seq) busy = false;
      }
    }, 60);
    return () => clearTimeout(timer);
  });

  onMount(() => {
    // Tauri delivers real file paths for drops, unlike the HTML5 drop event.
    const unlisten = getCurrentWebview().onDragDropEvent(({ payload }) => {
      if (payload.type === "enter" || payload.type === "over") dragging = true;
      else if (payload.type === "leave") dragging = false;
      else if (payload.type === "drop") {
        dragging = false;
        if (payload.paths[0]) openPath(payload.paths[0]);
      }
    });
    return () => void unlisten.then((fn) => fn());
  });
</script>

<div class="layout">
  <aside class="panel">
    <header class="brand"><img src="/icon.png" alt="" />ANSI Art</header>

    <section>
      <h2>Source</h2>
      <button class="primary" onclick={pickFile}>Open image…</button>
      {#if image}
        <p class="meta" title={image.path}>{image.name} · {image.width}×{image.height}</p>
      {:else}
        <p class="meta">…or drop a file anywhere</p>
      {/if}
    </section>

    <section>
      <h2>Render</h2>
      <label>
        Mode
        <select bind:value={opts.mode}>
          {#each MODES as m (m.value)}
            <option value={m.value}>{m.label}</option>
          {/each}
        </select>
        <small>{MODES.find((m) => m.value === opts.mode)?.hint}</small>
      </label>

      <Slider label="Width (columns)" bind:value={opts.width} min={8} max={240} />

      <label>
        Colors
        <select bind:value={opts.color}>
          {#each COLORS as c (c.value)}
            <option value={c.value}>{c.label}</option>
          {/each}
        </select>
      </label>

      {#if isGlyph}
        <label>
          Charset
          <select value={isCharset ? opts.glyph.charset : ""} onchange={(e) => (opts.glyph.charset = e.currentTarget.value)}>
            {#each CHARSETS as c (c)}
              <option value={c}>{c}</option>
            {/each}
            <option value="" disabled>custom</option>
          </select>
          <input
            type="text"
            placeholder="custom characters"
            value={isCharset ? "" : opts.glyph.charset}
            oninput={(e) => (opts.glyph.charset = e.currentTarget.value || "edges")}
          />
        </label>
        <label>
          Fill character
          <input type="text" maxlength="2" placeholder="none: matcher decides" bind:value={opts.glyph.fill} />
          <small>Used for solid areas, e.g. M, c, #, @, 0</small>
        </label>
        <Slider label="Solid at coverage" bind:value={opts.glyph.fillLevel} min={0.3} max={1} step={0.01} digits={2} />
        <Slider label="Flat colors" bind:value={opts.glyph.colors} min={1} max={8} />
      {:else}
        <label>
          Dithering
          <select bind:value={opts.dither} disabled={opts.color === "truecolor"}>
            {#each DITHERS as d (d.value)}
              <option value={d.value}>{d.label}</option>
            {/each}
          </select>
          {#if opts.color === "truecolor"}<small>Only applies to limited palettes</small>{/if}
        </label>
      {/if}

      {#if usesRamp}
        <label>
          Character ramp
          <select value={isPreset ? opts.ramp : ""} onchange={(e) => (opts.ramp = e.currentTarget.value)}>
            {#each RAMPS as r (r)}
              <option value={r}>{r}</option>
            {/each}
            <option value="" disabled>custom</option>
          </select>
          <input
            type="text"
            placeholder="custom, dark → bright"
            value={isPreset ? "" : opts.ramp}
            oninput={(e) => (opts.ramp = e.currentTarget.value || "standard")}
          />
        </label>
      {/if}

      {#if usesThreshold}
        <Slider label="Threshold" bind:value={opts.threshold} min={0} max={1} step={0.01} digits={2} />
      {/if}

      <Slider label="Cell aspect (w/h)" bind:value={opts.cellAspect} min={0.3} max={1} step={0.01} digits={2} />
    </section>

    <section>
      <h2>Adjust</h2>
      <Slider label="Brightness" bind:value={opts.adjust.brightness} min={-1} max={1} step={0.01} digits={2} />
      <Slider label="Contrast" bind:value={opts.adjust.contrast} min={0} max={3} step={0.01} digits={2} />
      <Slider label="Gamma" bind:value={opts.adjust.gamma} min={0.2} max={3} step={0.01} digits={2} />
      <Slider label="Saturation" bind:value={opts.adjust.saturation} min={0} max={2} step={0.01} digits={2} />
      <label class="check"><input type="checkbox" bind:checked={opts.adjust.invert} /> Invert</label>
      <button onclick={() => (opts.adjust = defaultOptions().adjust)}>Reset adjustments</button>
    </section>

    <section>
      <h2>Export</h2>
      <select bind:value={exportFormat}>
        {#each FORMATS as f (f.value)}
          <option value={f.value}>{f.label}</option>
        {/each}
      </select>
      <button class="primary" onclick={doExport} disabled={!image}>Export…</button>
    </section>
  </aside>

  <main>
    <div class="toolbar">
      <label class="inline">
        Terminal theme
        <select bind:value={themeName}>
          {#each Object.keys(THEMES) as name (name)}
            <option value={name}>{name}</option>
          {/each}
        </select>
      </label>
      <label class="inline">
        Font
        <input type="number" min="6" max="32" bind:value={fontSize} />
      </label>
      <label class="inline check"><input type="checkbox" bind:checked={preferWebgl} /> WebGL</label>
      <span class="spacer"></span>
      <span class="badge">{renderer}</span>
      {#if image}<span class="badge">{preview.cols}×{preview.rows}</span>{/if}
      {#if busy}<span class="badge busy">rendering…</span>{/if}
    </div>

    <div class="stage" style:background={THEMES[themeName].background}>
      {#if image}
        <Terminal
          ansi={preview.ansi}
          cols={preview.cols}
          rows={preview.rows}
          theme={THEMES[themeName]}
          {fontSize}
          {preferWebgl}
          bind:renderer
        />
      {:else}
        <button class="empty" onclick={pickFile}>Open or drop a PNG / JPG to start</button>
      {/if}
      {#if dragging}<div class="drop">Drop to load</div>{/if}
    </div>

    {#if message}
      <div class="message {message.kind}">
        <span>
          {message.text}
          {#if message.code}<br />Run: <code>{message.code}</code>{/if}
        </span>
        <button onclick={() => (message = null)}>×</button>
      </div>
    {/if}
  </main>
</div>
