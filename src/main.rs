use chip::{read_game, Machine};
use std::borrow::Cow;
use std::error::Error;
use std::io::Cursor;
use std::rc::Rc;
mod chip;
#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate glium;

use glium::{
    backend::Facade,
    glutin, index,
    texture::{ClientFormat, RawImage2d},
    Display, Surface, Texture2d,
};
use image::{jpeg::JpegDecoder, ImageDecoder};
use imgui::*;
use rand::prelude::*;

mod support;

struct CustomTexturesApp {
    my_texture_id: Option<TextureId>,
    machine: Machine,
}
fn generate_texture<F>(machine: &Machine, gl_ctx: &F) -> Texture2d
where
    F: Facade,
{
    const WIDTH: usize = 64;
    const HEIGHT: usize = 32;
    // Generate dummy texture
    let mut data = Vec::with_capacity(WIDTH * HEIGHT);

    for i in 0..HEIGHT {
        for j in 0..WIDTH {
            let pixel = machine.video_mem[HEIGHT - 1 - i][j];
            // let pixel:u8 = 0xFF;
            // let pixel:u8 = 0x00;
            data.push(pixel);
            data.push(pixel);
            data.push(pixel);
        }
    }

    let raw = RawImage2d {
        data: Cow::Owned(data),
        width: WIDTH as u32,
        height: HEIGHT as u32,
        format: ClientFormat::U8U8U8,
    };
    Texture2d::new(gl_ctx, raw).unwrap()
}

impl CustomTexturesApp {
    fn show_textures(&mut self, ui: &Ui) {
        Window::new(im_str!("Hello textures"))
            .size([400.0, 600.0], Condition::FirstUseEver)
            .build(ui, || {
                // if let Some(my_texture_id) = self.my_texture_id {
                //     Image::new(my_texture_id, [100.0, 100.0]).build(ui);
                // }
                for (index, line) in self.machine.get_source_code().iter().enumerate() {
                    ui.text(line);
                }
            });
    }
}

fn main() -> std::io::Result<()> {
    let buffer = read_game("INVADERS")?;

    let mut my_app = CustomTexturesApp {
        my_texture_id: None,
        machine: Machine::new(buffer.as_slice()),
    };

    let mut system = support::init(file!());

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            tex_coords: [f32; 2],
        }

        implement_vertex!(Vertex, position, tex_coords);

        glium::VertexBuffer::new(
            &system.display,
            &[
                Vertex {
                    position: [-1.0, -1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                    tex_coords: [1.0, 0.0],
                },
            ],
        )
        .unwrap()
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(
        &system.display,
        index::PrimitiveType::TriangleStrip,
        &[1 as u16, 2, 0, 3],
    )
    .unwrap();

    // compiling shaders and linking them together
    let program = program!(&system.display,
        140 => {
            vertex: "
                #version 140

                uniform mat4 matrix;

                in vec2 position;
                in vec2 tex_coords;

                out vec2 v_tex_coords;

                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",

            fragment: "
                #version 140
                uniform sampler2D tex;
                in vec2 v_tex_coords;
                out vec4 f_color;

                void main() {
                    f_color = texture(tex, v_tex_coords);
                }
            "
        },

        110 => {
            vertex: "
                #version 110

                uniform mat4 matrix;

                attribute vec2 position;
                attribute vec2 tex_coords;

                varying vec2 v_tex_coords;

                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",

            fragment: "
                #version 110
                uniform sampler2D tex;
                varying vec2 v_tex_coords;

                void main() {
                    gl_FragColor = texture2D(tex, v_tex_coords);
                }
            ",
        },

        100 => {
            vertex: "
                #version 100

                uniform lowp mat4 matrix;

                attribute lowp vec2 position;
                attribute lowp vec2 tex_coords;

                varying lowp vec2 v_tex_coords;

                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",

            fragment: "
                #version 100
                uniform lowp sampler2D tex;
                varying lowp vec2 v_tex_coords;

                void main() {
                    gl_FragColor = texture2D(tex, v_tex_coords);
                }
            ",
        },
    )
    .unwrap();
    system.main_loop(move |_, ui, display, renderer, target| {
        if !my_app.machine.stop {
            my_app.machine.cycle();
        }

        let opengl_texture = generate_texture(&my_app.machine, display.get_context());
        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            tex:
            glium::uniforms::Sampler::new(&opengl_texture)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };
        // let mut target = display.draw();
        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        // target.finish().unwrap();

        my_app.show_textures(ui)
    });
    Ok(())
}
