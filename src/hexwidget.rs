use egui::scroll_area::ScrollSource;
use egui_hooks::UseHookExt as _;
use std::ops::{Range, Rem};
use std::{collections::HashMap, sync::Arc};

use egui::Rect;
use egui::{Align2, Color32, FontId, Galley, Pos2, Sense, Ui, Vec2};

#[derive(Debug, Clone, Default)]
pub struct HexConfig {
    pub font: FontId,
    pub uppercase_hex: bool,
    pub byte_padding: f32,
    pub word_padding: f32,
    pub dword_padding: f32,
    pub qword_padding: f32,
}

#[derive(Default, Clone)]
pub struct HexState {
    galleys: HashMap<u8, Arc<Galley>>,
    config: HexConfig,
}

impl HexState {
    pub fn from_config(config: HexConfig) -> Self {
        HexState {
            config,
            ..Default::default()
        }
    }
}

pub trait ByteProvider {
    fn get_range(&self, range: Range<usize>) -> Option<&[u8]>;
    fn len(&self) -> usize;
}

impl ByteProvider for &[u8] {
    fn get_range(&self, mut range: Range<usize>) -> Option<&[u8]> {
        if range.is_empty() || range.start >= self.len() {
            return None;
        }
        range.end = range.end.min(self.len());
        Some(&self[range])
    }
    fn len(&self) -> usize {
        let this: &[u8] = self;
        this.len()
    }
}

pub fn draw_scroll<B: ByteProvider>(ui: &mut Ui, state: &mut HexState, data: B) {
    // TODO: rewrite offset/position calculations, this was just me learning to use egui
    let base_offset = ui.max_rect().min;
    let row_height = state.config.font.size;
    // remove all ui spacing for this context, ScrollArea uses it for row heights
    ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
    // NOTE: floating scroll is conflicting with egui_tiles, just disabled it for now
    ui.spacing_mut().scroll.floating = false;
    let max_visible_rows = (ui.available_height() / row_height) as usize;
    let hex_x_pos = base_offset.x + 6.0 * state.config.font.size;
    let ascii_x_pos = hex_x_pos + byte_pos(state, 15).x + state.config.font.size + 30.0;
    let total_width = ascii_x_pos + state.config.font.size * 16.0;
    let total_rows = data.len().div_ceil(16);
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .scroll_source(ScrollSource::SCROLL_BAR | ScrollSource::MOUSE_WHEEL)
        .show_rows(
            ui,
            row_height,
            total_rows + max_visible_rows,
            |ui, row_range| {
                let first_row = row_range.start;

                ui.allocate_rect(
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(total_width, row_height)),
                    Sense::hover(),
                );

                for row in row_range {
                    if row >= total_rows {
                        break;
                    }
                    let row_address = format!("{:08x}:", row * 16);
                    let row_pos = byte_pos(state, row * 16) + base_offset.to_vec2()
                        - Vec2::new(0.0, first_row as f32 * state.config.font.size);

                    ui.painter().text(
                        row_pos,
                        Align2::LEFT_TOP,
                        row_address,
                        state.config.font.clone(),
                        Color32::GRAY,
                    );
                    let row_start = row * 16;
                    let row_end = row_start + 16;
                    if let Some(row_data) = data.get_range(row_start..row_end) {
                        for (col, byte) in row_data.iter().enumerate() {
                            let offset = row * 16 + col;
                            let pos = byte_pos(state, offset)
                                - Vec2::new(0.0, first_row as f32 * state.config.font.size)
                                + Vec2::new(hex_x_pos, base_offset.y);
                            let (c, color) = if byte.is_ascii_graphic() {
                                (*byte as char, Color32::WHITE)
                            } else {
                                ('.', Color32::GRAY)
                            };
                            ui.painter().text(
                                Pos2::new(ascii_x_pos + col as f32 * state.config.font.size, pos.y),
                                Align2::LEFT_TOP,
                                c,
                                state.config.font.clone(),
                                color,
                            );
                            if state.galleys.is_empty() {
                                refresh_galleys(state, ui);
                            }
                            let galley = state.galleys.get(byte).unwrap();
                            let byte_color = if *byte == 0 {
                                Color32::GRAY
                            } else {
                                Color32::WHITE
                            };
                            ui.painter().galley_with_override_text_color(
                                pos,
                                galley.clone(),
                                byte_color,
                            );
                        }
                    }
                }
            },
        );
}

fn byte_pos(state: &HexState, index: usize) -> Pos2 {
    let line = index / 16;
    let column = index.rem(16);
    let words = column / 2;
    let dwords = column / 4;
    let qwords = column / 8;

    let cell_offset = column as f32 * state.config.font.size;
    let byte_offset = column as f32 * state.config.byte_padding;
    let word_offset = words as f32 * state.config.word_padding;
    let dword_offset = dwords as f32 * state.config.dword_padding;
    let qword_offset = qwords as f32 * state.config.qword_padding;
    let offset = cell_offset + byte_offset + word_offset + dword_offset + qword_offset;

    Pos2::new(offset, line as f32 * state.config.font.size)
}

pub fn refresh_galleys(state: &mut HexState, ui: &mut Ui) {
    let galleys = (0..=255)
        .map(|b: u8| {
            let s = ui.use_memo(
                || {
                    if state.config.uppercase_hex {
                        format!("{b:02X}")
                    } else {
                        format!("{b:02x}")
                    }
                },
                (b, state.config.uppercase_hex),
            );
            let galley = ui.painter().fonts_mut(|fonts| {
                fonts.layout_no_wrap(s, state.config.font.clone(), Color32::WHITE)
            });
            (b, galley)
        })
        .collect::<HashMap<_, _>>();
    state.galleys = galleys;
}
