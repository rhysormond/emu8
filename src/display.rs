use chip8::FrameBuffer;

/// # Chip-8 Display
/// The Chip-8 display is composed of 64x32 pixels black/white pixels.
pub struct Display {
    pub canvas: sdl2::render::WindowCanvas,
    width: usize,
    height: usize,
    resolution: usize,
}

impl Display {
    // TODO Error handling
    // TODO Better documentation
    pub fn new(sdl: &sdl2::Sdl, width: usize, height: usize, resolution: usize) -> Self {
        let video_subsystem = sdl.video().unwrap();
        let window = video_subsystem
            .window(
                "Emu-8",
                (width * resolution) as u32,
                (height * resolution) as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();

        let mut display = Display {
            canvas,
            width,
            height,
            resolution,
        };
        display.clear();
        display
    }

    /// Set color to black and clear
    pub fn clear(&mut self) {
        self.canvas
            .set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        self.canvas.clear();
    }

    /// Draw a pixel on the canvas
    pub fn draw_pixel(&mut self, x: usize, y: usize) {
        // Draw a rectangle based on the resolution
        self.canvas
            .set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        let px_rect = sdl2::rect::Rect::new(
            (x * self.resolution) as i32,
            (y * self.resolution) as i32,
            self.resolution as u32,
            self.resolution as u32,
        );
        self.canvas.fill_rect(px_rect).unwrap()
    }

    /// Draw a frame and render it
    pub fn render_frame(&mut self, frame: &FrameBuffer) {
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
