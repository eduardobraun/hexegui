use egui::{
    FontData, FontFamily, FontId,
    epaint::text::{FontInsert, FontPriority, InsertFontFamily},
};
use egui_tiles::Tree;

use crate::hexwidget::{HexConfig, HexState};

mod hexwidget;

struct Pane {
    nr: usize,
    // TODO: pane should have a way to define its content, currently every pane draws the same
    // hexwidget. Maybe this should be stored in egui::Memory
    data: Vec<u8>,
    hex_state: HexState,
}

struct TreeBehavior {}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        format!("Pane {}", pane.nr).into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        hexwidget::draw_scroll(ui, &mut pane.hex_state, pane.data.as_slice());
        egui_tiles::UiResponse::None
    }
}

fn create_tree() -> egui_tiles::Tree<Pane> {
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
    // TODO: this should be a file
    let data: Vec<u8> = (0..4098).map(|i| (i % 256) as u8).collect();
    let mut next_view_nr = 0;
    let mut gen_pane = || {
        let pane = Pane {
            nr: next_view_nr,
            data: data.clone(),
            hex_state: hex_state.clone(),
        };
        next_view_nr += 1;
        pane
    };

    let mut tiles = egui_tiles::Tiles::default();

    let mut tabs = vec![];
    tabs.push({
        let children = (0..7).map(|_| tiles.insert_pane(gen_pane())).collect();
        tiles.insert_horizontal_tile(children)
    });
    tabs.push({
        let cells = (0..11).map(|_| tiles.insert_pane(gen_pane())).collect();
        tiles.insert_grid_tile(cells)
    });
    tabs.push(tiles.insert_pane(gen_pane()));

    let root = tiles.insert_tab_tile(tabs);

    egui_tiles::Tree::new("my_tree", root, tiles)
}

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
            let tree = create_tree();
            Ok(Box::new(App { tree }))
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
    tree: Tree<Pane>,
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

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut behavior = TreeBehavior {};
            self.tree.ui(&mut behavior, ui);
        });
    }
}
