use chip8_core::{Chip8Error, Graphics};
use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window, Sdl};
use std::error::Error;

pub struct SdlGraphics {
    canvas: Canvas<Window>,
}

impl SdlGraphics {
    pub fn new(sdl_context: &Sdl) -> Result<SdlGraphics, Box<dyn Error>> {
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window("chip8", 640, 320)
            .position_centered()
            .opengl()
            .build()?;

        Ok(SdlGraphics {
            canvas: window.into_canvas().build()?,
        })
    }
}

impl Graphics for SdlGraphics {
    fn draw(&mut self, graphics: &[u8]) -> Result<(), Chip8Error> {
        let rects = graphics
            .iter()
            .enumerate()
            .filter(|(_, pixel)| **pixel == 1)
            .map(|(idx, _)| {
                let row = (idx / 64usize) * 10;
                let col = (idx % 64usize) * 10;
                Rect::new(col as i32, row as i32, 10, 10)
            })
            .collect::<Vec<Rect>>();

        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        if let Err(message) = self.canvas.fill_rects(&rects) {
            return Err(Chip8Error::GraphicsError(message));
        }
        self.canvas.present();

        Ok(())
    }
}
