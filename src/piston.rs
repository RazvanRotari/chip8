mod chip;
use chip::{read_game, Machine};

extern crate glutin_window;
extern crate graphics;
extern crate image;
extern crate opengl_graphics;
extern crate piston;
extern crate gfx_text;
extern crate piston_window;
#[macro_use]
extern crate lazy_static;

use graphics::rectangle::square;
use image::{imageops, Rgba, RgbaImage};
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston_window::{PistonWindow, TextureSettings};
use gfx_text;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    machine: Machine,
    exit: bool,
}

const WHITE: Rgba<u8> = Rgba::<u8>([0, 0, 0, 0xFF]);
const BLACK: Rgba<u8> = Rgba::<u8>([0xFF, 0xFF, 0xFF, 0xFF]);
impl App {
    fn render(&mut self, _window: &PistonWindow, args: &RenderArgs) {
        let width = 32;
        let height = 64;
        let scale = 5;
        use graphics::*;

        let settings = TextureSettings::new();
        let mut text = gfx_text::new(factory).build().unwrap();



        let image = Image::new().rect(square(0.0, 0.0, (height as f64) * scale as f64));
        let buffer = RgbaImage::from_fn(width, height, |x, y| {
            // BLACK
            // if self.machine.video_mem.len() >= (x * width + y) as usize {
            //     return BLACK;
            // }

            if self.machine.video_mem[x as usize][y as usize] == 0 {
                WHITE
            } else {
                BLACK
            }
        });
        let resized_image = imageops::resize(
            &buffer,
            width * scale,
            height * scale,
            imageops::FilterType::Triangle,
        );
        let texture = Texture::from_image(&resized_image, &settings);

        self.gl.draw(args.viewport(), |c, gl| {
            clear(color::WHITE, gl);

            image.draw(&texture, &DrawState::new_alpha(), c.transform, gl);
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.exit = self.machine.cycle();
    }
}

fn piston() -> std::io::Result<()> {
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("piston: image", [100, 100])
        .exit_on_esc(true)
        .graphics_api(opengl)
        .build()
        .unwrap();

    let buffer = read_game("GUESS")?;
    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        machine: Machine::new(buffer.as_slice()),
        exit: false,
    };

    let source = app.machine.get_source_code();
    println!("{}", source);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&window, &args);
        }

        if let Some(args) = e.update_args() {
            if app.exit {
                continue;
            }
            app.update(&args);
        }
    }
    Ok(())
}
