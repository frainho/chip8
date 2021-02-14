use chip8_core::{Chip8Error, Graphics};
use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window, Sdl};
use std::error::Error;

pub struct SdlGraphics {
    canvas: Canvas<Window>,
}

impl SdlGraphics {
    const WIDTH: u32 = 640;
    const HEIGHT: u32 = 320;
    const SCALE: u32 = 10;

    pub fn new(sdl_context: &Sdl) -> Result<SdlGraphics, Box<dyn Error>> {
        let canvas = sdl_context
            .video()?
            .window("chip8", Self::WIDTH, Self::HEIGHT)
            .position_centered()
            .opengl()
            .build()?
            .into_canvas()
            .build()?;

        Ok(SdlGraphics { canvas })
    }
}

impl Graphics for SdlGraphics {
    fn draw(&mut self, graphics: &[u8]) -> Result<(), Chip8Error> {
        let rects = graphics
            .iter()
            .enumerate()
            .filter(|(_, pixel)| **pixel == 1)
            .map(|(idx, _)| {
                let idx = idx as u32;
                let row = (idx / 64) * Self::SCALE;
                let col = (idx % 64) * Self::SCALE;
                Rect::new(col as i32, row as i32, Self::SCALE, Self::SCALE)
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
