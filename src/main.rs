use egui::{
    FontData, FontFamily, FontId,
    epaint::text::{FontInsert, FontPriority, InsertFontFamily},
};

use crate::hexwidget::{HexConfig, HexState};

mod hexwidget;

fn main() {
    {
        // Silence wgpu log spam (https://github.com/gfx-rs/wgpu/issues/3206)
        let mut rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| {
            if cfg!(debug_assertions) {
                "debug".to_owned()
            } else {
                "info".to_owned()
            }
        });
        for loud_crate in ["naga", "wgpu_core", "wgpu_hal"] {
            if !rust_log.contains(&format!("{loud_crate}=")) {
                rust_log += &format!(",{loud_crate}=warn");
            }
        }

        // SAFETY: we call this from the main thread without any other threads running.
        #[expect(unsafe_code)]
        unsafe {
            std::env::set_var("RUST_LOG", rust_log);
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 1024.0])
            .with_drag_and_drop(true),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    let result = eframe::run_native(
        "egui demo app",
        options,
        Box::new(|cc| {
            egui_font_loader::load_font!(
                cc.egui_ctx,
                "IosevkaNerdFontMono-Light",
                "../resources/fonts/IosevkaNerdFontMono-Light.ttf"
            );
            let config = HexConfig {
                font: FontId {
                    size: 20.0,
                    family: FontFamily::Name("IosevkaNerdFontMono-Light".into()),
                },
                uppercase_hex: false,
                byte_padding: 2.0,
                word_padding: 2.0,
                dword_padding: 2.0,
                qword_padding: 6.0,
            };
            let hex_state = HexState::from_config(config);
            Ok(Box::new(App { hex_state }))
        }),
    );

    match result {
        Ok(()) => {}
        Err(err) => {
            // This produces a nicer error message than returning the `Result`:
            // print_error_and_exit(&err);
            tracing::error!(error=%err);
        }
    }
}

struct App {
    hex_state: HexState,
}

impl eframe::App for App {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        // Give the area behind the floating windows a different color, because it looks better:
        let color = egui::lerp(
            egui::Rgba::from(visuals.panel_fill)..=egui::Rgba::from(visuals.extreme_bg_color),
            0.5,
        );
        let color = egui::Color32::from(color);
        color.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
            let fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
        }

        let data: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();
        egui::panel::CentralPanel::default()
            .frame(egui::Frame::new().inner_margin(4))
            .show(ctx, |ui| {
                hexwidget::draw_scroll(ui, &mut self.hex_state, data.as_slice());
            });
    }
}
