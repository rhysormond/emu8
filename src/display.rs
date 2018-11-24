use chip8::FrameBuffer;

/// # Display
///
/// The Chip-8 display is composed of 64x32 pixels black/white pixels.
/// The on/off state of these pixels is encoded as 1/0 respectively in a 2d array of 64x32 bits.
/// The display only gets a call to `render` when the Chip-8 FrameBuffer is updated.
pub trait Display
where
    Self: std::marker::Sized,
{
    /// Creates a new display object bound to an sdl2 context.
    ///
    /// # Arguments
    /// * `sdl` an sdl2 context with which to draw.
    /// * `width` the horizontal size of the display measured in pixels.
    /// * `height` the vertical size of the display measured in pixels.
    /// * `scale` the magnitude with which that size of each pixel should be multiplied.
    fn new(sdl: &sdl2::Sdl, width: usize, height: usize, scale: usize) -> Self;

    /// Clears the entire display by setting every pixel to black.
    fn clear(&mut self);

    /// Renders a single Chip-8 FrameBuffer.
    ///
    /// Individual pixels from the `frame` are drawn to the display and the entire
    ///
    /// # Arguments
    /// * `frame` a Chip-8 FrameBuffer that represents the state of every pixel on the Display.
    fn render(&mut self, frame: &FrameBuffer);
}

pub struct Window {
    pub canvas: sdl2::render::WindowCanvas,
    width: usize,
    height: usize,
    scale: usize,
}

impl Window {
    /// Draws a single Chip-8 pixel on the display, scaled appropriately.
    ///
    /// # Arguments
    /// * `x` the x axis position of the pixel in Chip-8 space (before scaling).
    /// * `y` the y axis position of the pixel in Chip-8 space (before scaling).
    fn draw_pixel(&mut self, x: usize, y: usize) {
        self.canvas
            .set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        let px_rect = sdl2::rect::Rect::new(
            (x * self.scale) as i32,
            (y * self.scale) as i32,
            self.scale as u32,
            self.scale as u32,
        );
        self.canvas
            .fill_rect(px_rect)
            .unwrap();
    }
}

impl Display for Window {
    fn new(sdl: &sdl2::Sdl, width: usize, height: usize, scale: usize) -> Self {
        let video_subsystem = sdl.video().unwrap();
        let window = video_subsystem
            .window("Emu-8", (width * scale) as u32, (height * scale) as u32)
            .position_centered()
            .opengl()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();

        let mut display = Window {
            canvas,
            width,
            height,
            scale,
        };

        display.clear();
        display
    }

    fn clear(&mut self) {
        self.canvas
            .set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        self.canvas.clear();
    }

    fn render(&mut self, frame: &FrameBuffer) {
        self.clear();
        for x in 0..self.width {
            for y in 0..self.height {
                // If the pixel is on, draw it
                if frame[x][y] == 1 {
                    self.draw_pixel(x, y)
                }
            }
        }
        self.canvas.present()
    }
}

#[cfg(test)]
mod test_window {
    // TODO figure out how to inspect state of sdl2 canvas and write tests
}
