use sdl2::pixels::{PixelFormatEnum, Color};
use sdl2::render::Canvas;
use sdl2::video::{Window, WindowContext};
use sdl2::render::{Texture, TextureCreator};

const TILE_MAP_SIZE: u16 = 1024;
const ORIGINAL_GB_DISPLAY_WIDTH: u32 = 160;
const ORIGINAL_GB_DISPLAY_HEIGHT: u32 = 144;
const SCALING_FACTOR: u32 = 7;
// The display is 32x32 tiles. Each tile is 8x8 pixels, since we are using
// PixelFormatEnum::RGB888, each pixel occupies 3 bytes, so the required memory for displaying all
// tiles is: ((32 * 8) * (32 * 8)) * 3 = 196_608 bytes
const DISPLAY_SIZE: u32 = 196_608;

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
            .window(title.as_str(), ORIGINAL_GB_DISPLAY_WIDTH * SCALING_FACTOR, ORIGINAL_GB_DISPLAY_HEIGHT * SCALING_FACTOR)
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
}

pub struct GPU {
    sdl_utils: SdlUtils,
    tile_map: [u8; TILE_MAP_SIZE as usize],
    display: [u8; DISPLAY_SIZE as usize],
}

impl GPU {
    pub fn new() -> Self {
        Self {
            sdl_utils: SdlUtils::new(),
            tile_map: [0; TILE_MAP_SIZE as usize],
            display: [0xFF; DISPLAY_SIZE as usize]
        }
    }
}

impl Drawable for GPU {
    // TODO (luizf): Don't create texture in every call to this function
    fn draw(&mut self) {
        let mut texture = self.sdl_utils.texture_creator
                .create_texture_streaming(PixelFormatEnum::RGB888, 256, 256)
                .expect("Couldn't create texture");
        let _ = texture.update(None, &self.display, ORIGINAL_GB_DISPLAY_WIDTH as usize);
        let _ = self.sdl_utils.canvas.copy(&texture, None, None);
        self.sdl_utils.canvas.present(); 
    }
}
