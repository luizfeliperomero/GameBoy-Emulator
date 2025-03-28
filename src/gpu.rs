use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::Canvas;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

const TILE_MAP_SIZE: u16 = 1024;
const ORIGINAL_GB_DISPLAY_WIDTH: u32 = 160;
const ORIGINAL_GB_DISPLAY_HEIGHT: u32 = 144;
const SCALING_FACTOR: u32 = 7;
// The display is 32x32 tiles. Each tile is 8x8 pixels, since we are using
// PixelFormatEnum::RGB888, each pixel occupies 3 bytes, so the required memory for displaying all
// tiles is: ((32 * 8) * (32 * 8)) * 3 = 196_608 bytes
const DISPLAY_SIZE_IN_BYTES: u32 = 196_608;

struct SdlUtils {
    pub canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
}

impl SdlUtils {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let title = "GameBoy Emulator".to_string();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(
                title.as_str(),
                ORIGINAL_GB_DISPLAY_WIDTH * SCALING_FACTOR,
                ORIGINAL_GB_DISPLAY_HEIGHT * SCALING_FACTOR,
            )
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        let _ = canvas.set_logical_size(ORIGINAL_GB_DISPLAY_WIDTH, ORIGINAL_GB_DISPLAY_HEIGHT);
        let texture_creator = canvas.texture_creator();

        Self {
            canvas,
            texture_creator,
        }
    }
}

pub trait Drawable {
    fn draw(&mut self);
    fn map_tile_pixels(&self, tile: &[u8; 16]) -> [u8; 192];
    fn arrange_tile_bytes(&self, tile: &[u8]) -> [u8; 16];
    fn extract_low_bits(&self, byte: u8) -> u8;
    fn extract_high_bits(&self, byte: u8) -> u8;
    fn set_tile_on_display(&mut self, tile: &[u8; 192], position: usize);
}

pub struct GPU {
    sdl_utils: SdlUtils,
    display: [u8; DISPLAY_SIZE_IN_BYTES as usize],
}

impl GPU {
    pub fn new() -> Self {
        Self {
            sdl_utils: SdlUtils::new(),
            display: [0x0; DISPLAY_SIZE_IN_BYTES as usize],
        }
    }
}

impl Drawable for GPU {
    // TODO (luizf): Don't create texture in every call to this function
    fn draw(&mut self) {
        let mut texture = self
            .sdl_utils
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB888, 128, 128)
            .expect("Couldn't create texture");
        let _ = texture.update(None, &self.display, ORIGINAL_GB_DISPLAY_WIDTH as usize);
        let _ = self.sdl_utils.canvas.copy(&texture, None, None);
        self.sdl_utils.canvas.present();
    }
    fn arrange_tile_bytes(&self, tile: &[u8]) -> [u8; 16] {
        let mut result: [u8; 16] = [0; 16];
        tile.chunks(2) 
            .enumerate()
            .for_each(|(i, chunk)| {
                if let [lhs, rhs] = chunk {
                    let low = (self.extract_high_bits(*lhs) << 4) | self.extract_high_bits(*rhs);
                    let high = (self.extract_low_bits(*lhs) << 4) | self.extract_low_bits(*rhs);
                    result[i * 2] = high;
                    result[i * 2 + 1] = low;
                } 
            });
        result
    } 
    fn set_tile_on_display(&mut self, tile: &[u8; 192], position: usize) {
        tile.chunks(3)
            .enumerate()
            .for_each(|(i, rgb)| {
                let display_index = position + (i * 3);
                self.display[display_index] = rgb[0];
                self.display[display_index + 1] = rgb[1];
                self.display[display_index + 2] = rgb[2];
            })
    }
    fn map_tile_pixels(&self, tile: &[u8; 16]) -> [u8; 192]{
        let mut result_tile: [u8; 192] = [0; 192];
        tile.iter()
            .enumerate()
            .for_each(|(i, b)| {
                for j in (0..8).step_by(2) {
                    let mask = 0b11 << (6 - j);
                    let offset = 6 - j;
                    let result = (b & mask) >> offset;
                    let color: u32 = match result {
                        0 => 0xE0F8D0,
                        1 => 0x89C06F,
                        2 => 0x356856,
                        3 => 0x081820,
                        _ => 0x000000,
                    };
                    let index = i * 3;
                    result_tile[index as usize] = ((color & 0x00_FF_00_00) >> 16) as u8;
                    result_tile[(index + 1) as usize] = ((color & 0x00_00_FF_00) >> 8) as u8;
                    result_tile[(index + 2) as usize] = (color & 0x00_00_00_FF) as u8;
                }
            });
        result_tile
    }
    fn extract_high_bits(&self, byte: u8) -> u8 {
        let mut result = 0;
        for i in (0..8).step_by(2) {
            let high_bit = (byte >> (6 - i)) & 0b10;
            result = (result << 1) | (high_bit >> 1);
        }
        result
    }
    fn extract_low_bits(&self, byte: u8) -> u8 {
        let mut result = 0;
        for i in (0..8).step_by(2) {
            let low_bit = (byte >> (6 - i)) & 0b01;
            result = (result << 1) | low_bit;
        }
        result
    }
}
