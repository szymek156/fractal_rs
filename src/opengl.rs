use crate::fractal::OutBuffer;
use glium::index::NoIndices;
use glium::{glutin, Surface, VertexBuffer};
use glium::{glutin::dpi::LogicalSize, glutin::event_loop::EventLoop, Display};
use image::GenericImage;
use lazy_static::lazy_static;
use std::sync::RwLock;

use std::sync::mpsc::Receiver;
use std::time::Duration;

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

fn get_texture(
    display: &Display,
    receiver: &Receiver<OutBuffer>,
) -> Option<glium::texture::Texture2d> {
    let fractal = match receiver.try_recv() {
        Ok(buff) => buff,
        Err(_) => return None,
    };

    let dimensions = fractal.dimensions();

    let image = glium::texture::RawImage2d::from_raw_rgb(fractal.clone().into_raw(), dimensions);

    let texture = glium::texture::Texture2d::new(display, image).unwrap();

    Some(texture)
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

pub fn run(receiver: Receiver<OutBuffer>) {
    let event_loop = EventLoop::new();

    let display = glium::Display::new(
        glutin::window::WindowBuilder::new().with_inner_size(LogicalSize::new(800, 800)),
        glutin::ContextBuilder::new(),
        &event_loop,
    )
    .unwrap();

    let plane = create_plane(&display);

    let program = create_program(&display);

    let mut fps_count = 0;
    let mut fps_measure = std::time::Instant::now() + std::time::Duration::from_secs(1);

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

        if let Some(texture) = get_texture(&display, &receiver) {
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 1.0, 1.0);

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
        }

        if fps_measure < next_frame_time {
            fps_measure = next_frame_time + std::time::Duration::from_secs(1);

            println!("FPS {}", fps_count);
            fps_count = 0;
        } else {
            fps_count += 1;
        }
    });
}
