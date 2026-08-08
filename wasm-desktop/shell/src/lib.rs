use eframe::egui;
use wasm_bindgen::prelude::*;

mod desktop;
mod fs_seed;
mod gradient;
mod process;

use desktop::Desktop;

/// Entry point called from the host page's JS. Boots the eframe/egui
/// WebRunner onto the given <canvas> element.
#[wasm_bindgen]
pub fn start(canvas: web_sys::HtmlCanvasElement) -> js_sys::Promise {
    console_error_panic_hook::set_once();

    wasm_bindgen_futures::future_to_promise(async move {
        let web_options = eframe::WebOptions::default();
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(Desktop::new()))),
            )
            .await
            .map(|_| JsValue::NULL)
            .map_err(|err| JsValue::from_str(&format!("{err:?}")))
    })
}

impl eframe::App for Desktop {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.draw(ui, frame);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Fully transparent — the animated GLSL wallpaper repaints the
        // whole viewport every frame anyway, so this is never visible.
        [0.0, 0.0, 0.0, 0.0]
    }
}
