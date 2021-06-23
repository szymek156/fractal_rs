use std::io::Cursor;

use glium::backend::Facade;
use glium::index::NoIndices;
use glium::{glutin, Surface, VertexBuffer};
use glium::{glutin::event_loop::EventLoop, Display};
use image::Rgba;
use num_complex::Complex;
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

struct Plane {
    vertex_buffer: VertexBuffer<Vertex>,
    indices: NoIndices,
}
implement_vertex!(Vertex, position, tex_coords);

fn get_texture(display: &Display) -> glium::texture::Texture2d {
    // Oh lol:
    // https://crates.io/crates/image
    let imgx = 800;
    let imgy = 800;


    let scalex = 3.0 / imgx as f32;
    let scaley = 3.0 / imgy as f32;
    
    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (0.3 * x as f32) as u8;
        let b = (0.3 * y as f32) as u8;
        *pixel = image::Rgb([r, 0, b]);

        let cx = y as f32 * scalex - 1.5;
        let cy = x as f32 * scaley - 1.5;

        let c = Complex::new(-0.4f32, 0.6f32);
        let mut z = Complex::new(cx, cy);

        let mut i = 0;
        while i < 255 && z.norm() <= 2.0 {
            z = z * z + c;
            i += 1;
        }

        let image::Rgb(data) = *pixel;
        *pixel = image::Rgb([data[0], i as u8, data[2]]);
    }

    let image = glium::texture::RawImage2d::from_raw_rgb(imgbuf.into_raw(), (imgx, imgy));

    let texture = glium::texture::Texture2d::new(display, image).unwrap();

    texture
}

fn create_plane(display: &Display) -> Plane {
    let shape = vec![
        Vertex {
            position: [-1.0, 1.0],
            tex_coords: [0.0, 1.0],
        },
        Vertex {
            position: [1.0, 1.0],
            tex_coords: [1.0, 1.0],
        },
        Vertex {
            position: [-1.0, -1.0],
            tex_coords: [0.0, 0.0],
        },
        Vertex {
            position: [1.0, -1.0],
            tex_coords: [1.0, 0.0],
        },
    ];

    Plane {
        vertex_buffer: glium::VertexBuffer::new(display, &shape).unwrap(),
        indices: glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
    }
}

fn create_program(display: &Display) -> glium::Program {
    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;

        uniform mat4 matrix;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;
        out vec4 color;

        uniform sampler2D tex;

        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;

    let program =
        glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

    program
}

pub fn run() {
    let event_loop = glutin::event_loop::EventLoop::new();

    let display = glium::Display::new(
        glutin::window::WindowBuilder::new(),
        glutin::ContextBuilder::new(),
        &event_loop,
    )
    .unwrap();

    let plane = create_plane(&display);

    let program = create_program(&display);

    event_loop.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let texture = get_texture(&display);

        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ 0.0, 0.0, 0.0, 1.0f32],
            ],
            tex: &texture,
        };

        target
            .draw(
                &plane.vertex_buffer,
                &plane.indices,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();
    });
}
