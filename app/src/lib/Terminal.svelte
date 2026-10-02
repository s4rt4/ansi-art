<script lang="ts" module>
  export type Renderer = "webgl" | "dom";
</script>

<script lang="ts">
  // Real terminal emulator for the preview, so what you see is what a terminal shows.
  // Renderer: WebGL when it works, DOM otherwise. WebGL on WebKitGTK can fail to
  // start or lose its context (GPU driver resets), so both cases fall back to DOM.

  import { onMount } from "svelte";
  import { Terminal, type ITheme } from "@xterm/xterm";
  import { WebglAddon } from "@xterm/addon-webgl";
  import "@xterm/xterm/css/xterm.css";

  interface Props {
    ansi: string;
    cols: number;
    rows: number;
    theme: ITheme;
    fontSize?: number;
    preferWebgl?: boolean;
    renderer?: Renderer;
  }

  let {
    ansi,
    cols,
    rows,
    theme,
    fontSize = 14,
    preferWebgl = true,
    renderer = $bindable("dom"),
  }: Props = $props();

  let host: HTMLDivElement;
  let term = $state.raw<Terminal>();
  let webgl: WebglAddon | undefined;

  const FONT_STACK = '"Cascadia Mono", "JetBrains Mono", "DejaVu Sans Mono", "Noto Sans Mono", Menlo, Consolas, monospace';

  onMount(() => {
    const t = new Terminal({
      fontFamily: FONT_STACK,
      fontSize,
      lineHeight: 1,
      letterSpacing: 0,
      // Draw block/box glyphs geometrically so they tile without seams, whatever the font.
      customGlyphs: true,
      convertEol: true,
      disableStdin: true,
      cursorBlink: false,
      cursorInactiveStyle: "none",
      scrollback: 0,
      theme,
      cols: Math.max(cols, 2),
      rows: Math.max(rows, 1),
    });
    t.open(host);
    term = t;
    return () => {
      disableWebgl();
      t.dispose();
    };
  });

  function enableWebgl(t: Terminal) {
    try {
      const addon = new WebglAddon();
      addon.onContextLoss(() => {
        console.warn("[terminal] WebGL context lost, falling back to DOM renderer");
        disableWebgl();
      });
      t.loadAddon(addon);
      webgl = addon;
      renderer = "webgl";
    } catch (err) {
      console.warn("[terminal] WebGL unavailable, using DOM renderer", err);
      disableWebgl();
    }
  }

  function disableWebgl() {
    webgl?.dispose();
    webgl = undefined;
    renderer = "dom";
  }

  $effect(() => {
    if (!term) return;
    if (preferWebgl && !webgl) enableWebgl(term);
    else if (!preferWebgl && webgl) disableWebgl();
  });

  $effect(() => {
    if (term) term.options.theme = theme;
  });

  $effect(() => {
    if (term) term.options.fontSize = fontSize;
  });

  $effect(() => {
    if (!term) return;
    term.resize(Math.max(cols, 2), Math.max(rows, 1));
    term.reset();
    // Hide the cursor, and drop the final newline so the last row doesn't scroll away.
    term.write("\x1b[?25l" + ansi.replace(/\n$/, ""));
  });
</script>

<div class="term" bind:this={host}></div>

<style>
  .term {
    display: inline-block;
  }
  .term :global(.xterm-viewport) {
    overflow: hidden !important;
  }
</style>
