mod chip8;
mod font_set;
mod test_utils;
mod types;

use std::{error::Error, thread, time::Duration};

use chip8::Chip8;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

fn main() -> Result<(), Box<dyn Error>> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("chip8", 640, 320)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut chip8 = Chip8::new(Box::new(rand::thread_rng()));
    chip8.initialize();
    chip8.load_program("test_opcode.ch8")?;

    'main: loop {
        chip8.emulate_cycle();
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                _ => {}
            }
        }

        let rects = chip8
            .graphics
            .iter()
            .enumerate()
            .filter(|(_, pixel)| **pixel == 1)
            .map(|(idx, _)| {
                let row = (idx / 64usize) * 10;
                let col = (idx % 64usize) * 10;
                Rect::new(col as i32, row as i32, 10, 10)
            })
            .collect::<Vec<Rect>>();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.fill_rects(&rects)?;
        canvas.present();
        thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
