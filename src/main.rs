use rchip_8::emulator::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::env;
use std::fs::File;
use std::io::Read;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 10;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    // Initialize emulator and load ROM
    let mut chip8 = Emulator::new();
    let mut rom = File::open(&args[1]).expect("Could not open ROM file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    chip8.load_game(&buffer);

    // Initialize SDL2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Chip 8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Main loop
    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'gameloop,

                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key_convert(key) {
                        chip8.keypress(k, true);
                    }
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key_convert(key) {
                        chip8.keypress(k, false);
                    }
                }

                _ => (),
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            chip8.cpu_cycle();
        }

        chip8.tick_timer();
        draw_screen(&chip8, &mut canvas);
    }
}

/// Draws the CHIP-8 display to the SDL2 canvas
fn draw_screen(emu: &Emulator, canvas: &mut Canvas<Window>) {
    // Clear canvas
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    // Collect rectangles for all pixels that are ON
    let rects: Vec<Rect> = emu
        .get_display()
        .iter()
        .copied() // convert &bool -> bool
        .enumerate()
        .filter(|(_, p)| *p)
        .map(|(i, _)| {
            let x = (i % SCREEN_WIDTH) as i32;
            let y = (i / SCREEN_WIDTH) as i32;
            Rect::new(x * SCALE as i32, y * SCALE as i32, SCALE, SCALE)
        })
        .collect();

    // Draw all white pixels at once
    canvas.set_draw_color(Color::WHITE);
    let _ = canvas.fill_rects(&rects);

    canvas.present();
}

fn key_convert(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),

        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),

        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),

        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),

        _ => None,
    }
}
