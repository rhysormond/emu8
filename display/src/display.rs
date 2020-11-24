use sdl2::pixels::PixelFormatEnum;

use core::constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use core::state::FrameBuffer;

const SCALE: usize = 10;

/// # Display
/// The Chip-8 display is composed of 64x32 pixels black/white pixels.
/// The on/off state of these pixels is encoded as 1/0 respectively in a 2d array of 64x32 bits.
/// The display only gets a call to `render` when the Chip-8 FrameBuffer is updated.
pub struct Display {
    canvas: sdl2::render::WindowCanvas,
    width: usize,
    height: usize,
}

// TODO handle errors better
impl Display {
    /// Creates a new display object bound to an sdl2 context.
    ///
    /// # Arguments
    /// * `sdl` an sdl2 context with which to draw
    /// * `width` the horizontal size of the display measured in pixels
    /// * `height` the vertical size of the display measured in pixels
    /// * `scale` the size multiplier for each pixel
    pub fn new(sdl: &sdl2::Sdl) -> Self {
        let video_subsystem = sdl.video().unwrap();
        let window = video_subsystem
            .window(
                "Emu-8",
                (DISPLAY_WIDTH * SCALE) as u32,
                (DISPLAY_HEIGHT * SCALE) as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();

        Display {
            canvas,
            width: DISPLAY_WIDTH,
            height: DISPLAY_HEIGHT,
        }
    }

    /// Formats a Chip-8 FrameBuffer for rendering as an SDL2 texture.
    ///
    /// An SDL2 texture is a 1D array of ints that represent concatenated rows of RGB pixels.
    ///
    /// This creates a black and white rendering by:
    /// - Flattening the 2D frame buffer into a 1D array by concatenating its rows
    /// - Triplicating each element of that 1D array to represent the RGB values of each pixel
    /// - Multiplying each value by 255 to convert from a binary state to 0-255 intensity
    ///
    /// # Arguments
    /// * `frame` a Chip-8 FrameBuffer
    fn frame_to_sdl_texture(frame: &FrameBuffer) -> Vec<u8> {
        frame
            .iter()
            .flat_map(|a| a.iter())
            .flat_map(|a| std::iter::repeat(a).take(3))
            .map(|a| a * 255)
            .collect()
    }

    /// Formats the Chip-8 FrameBuffer as an SDL2 RGB24 texture and renders it.
    ///
    /// # Arguments
    /// * `frame` a Chip-8 FrameBuffer
    pub fn render(&mut self, frame: &FrameBuffer) {
        let texture_creator = self.canvas.texture_creator();

        let mut texture = texture_creator
            .create_texture_streaming(
                PixelFormatEnum::RGB24,
                self.width as u32,
                self.height as u32,
            )
            .unwrap();

        texture
            .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                buffer.copy_from_slice(&Display::frame_to_sdl_texture(frame));
            })
            .unwrap();

        self.canvas.copy(&texture, None, None).unwrap();
        self.canvas.present()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_to_sdl_texture() {
        let mut frame: FrameBuffer = [[0; 64]; 32];
        frame[0][0..2].copy_from_slice(&[0, 1]);
        frame[1][0..2].copy_from_slice(&[1, 0]);
        let frame = Display::frame_to_sdl_texture(&frame);

        let mut expected: Vec<u8> = vec![0; 6144];
        expected[0..6].copy_from_slice(&[0, 0, 0, 255, 255, 255]);
        expected[192..198].copy_from_slice(&[255, 255, 255, 0, 0, 0]);

        assert_eq!(frame, expected);
    }
}
