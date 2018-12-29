use chip8::{FrameBuffer, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};

/// # Display
///
/// The Chip-8 display is composed of 64x32 pixels black/white pixels.
/// The on/off state of these pixels is encoded as 1/0 respectively in a 2d array of 64x32 bits.
/// The display only gets a call to `render` when the Chip-8 FrameBuffer is updated.
pub struct Display {
    pub canvas: WindowCanvas,
    width: usize,
    height: usize,
    scale: usize,
}

impl Display {
    /// Creates a new display object bound to an sdl2 context.
    ///
    /// # Arguments
    /// * `sdl` an sdl2 context with which to draw.
    /// * `width` the horizontal size of the display measured in pixels.
    /// * `height` the vertical size of the display measured in pixels.
    /// * `scale` the magnitude with which that size of each pixel should be multiplied.
    pub fn new(sdl: &sdl2::Sdl, width: usize, height: usize, scale: usize) -> Self {
        let video_subsystem = sdl.video().unwrap();
        let window = video_subsystem
            .window("Emu-8", (width * scale) as u32, (height * scale) as u32)
            .position_centered()
            .opengl()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();

        let mut display = Display {
            canvas,
            width,
            height,
            scale,
        };

        display.render(&[[0; 32]; 64]);
        display
    }

    /// Renders a single Chip-8 FrameBuffer.
    ///
    /// Individual pixels from the `frame` are drawn to the display and the entire
    ///
    /// # Arguments
    /// * `frame` a Chip-8 FrameBuffer that represents the state of every pixel on the Display.
    pub fn render(&mut self, frame: &FrameBuffer) {
        let texture_creator = self.canvas.texture_creator();

        let mut texture = texture_creator
            .create_texture_streaming(
                PixelFormatEnum::RGB24,
                DISPLAY_WIDTH as u32,
                DISPLAY_HEIGHT as u32,
            )
            .unwrap();

        // TODO clean up this logic
        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for (x, row) in frame.iter().enumerate() {
                    for (y, pixel) in row.iter().enumerate() {
                        let offset = y * pitch + x * 3;
                        let color = *pixel * 255;
                        buffer[offset..offset + 3].copy_from_slice(&[color, color, color]);
                    }
                }
            })
            .unwrap();

        self.canvas.copy(&texture, None, None).unwrap();
        self.canvas.present()
    }
}

#[cfg(test)]
mod test_window {
    // TODO figure out how to inspect state of sdl2 canvas and write tests
}
