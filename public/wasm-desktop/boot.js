import init, { start } from "./wasm_desktop.js";

// eframe's own WebRunner installs a ResizeObserver on the canvas that
// keeps its backing-buffer size in sync with its CSS box size and the
// real devicePixelRatio (see egui/eframe web_runner + events.rs). We must
// NOT also set canvas.width/height/style ourselves — a previous version
// of this file did, and the two resize systems fought over the same
// canvas every frame/resize, causing stutter and a mismatched/blurry
// backing buffer. The canvas's CSS size (width:100%; height:100%) is
// already set in desktop.astro; that's all eframe needs.
async function boot() {
  const canvas = document.getElementById("desktop-canvas");
  const bootScreen = document.getElementById("desktop-boot");
  if (!canvas) return;

  try {
    await init();
    await start(canvas);
    bootScreen?.remove();
  } catch (err) {
    console.error("[desktop] failed to boot wasm module", err);
    const label = bootScreen?.querySelector(".boot-label");
    if (label) label.textContent = "Failed to load desktop mode. Try refreshing.";
  }
}

boot();
