use std::io::Cursor;

use glium::index::NoIndices;
use glium::{glutin, Surface, VertexBuffer};
use glium::{glutin::event_loop::EventLoop, Display};
pub struct Context {
    event_loop: EventLoop<()>,
    display: Display,
}

impl Context {
    fn new() -> Self {
        let wb = glutin::window::WindowBuilder::new();
        let cb = glutin::ContextBuilder::new();

        let event_loop = glutin::event_loop::EventLoop::new();
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

        Context {
            event_loop: event_loop,
            display: display,
        }
    }
}

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

fn create_texture(context: &Context) -> glium::texture::Texture2d {
    let image = image::load(
        Cursor::new(&include_bytes!("opengl.png")),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let image_dimensions = image.dimensions();
    let image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let texture = glium::texture::Texture2d::new(&context.display, image).unwrap();

    texture
}

fn create_plane(context: &Context) -> Plane {
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
        vertex_buffer: glium::VertexBuffer::new(&context.display, &shape).unwrap(),
        indices: glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
    }
}

fn create_program(context: &Context) -> glium::Program {
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

    let program = glium::Program::from_source(
        &context.display,
        vertex_shader_src,
        fragment_shader_src,
        None,
    )
    .unwrap();

    program
}

pub fn init() -> Context {
    #[allow(unused_imports)]
    let context = Context::new();

    let texture = create_texture(&context);

    let plane = create_plane(&context);

    let program = create_program(&context);

    context
}


pub fn run(context : &Context) {
    context.event_loop.run(move |event, _, control_flow| {
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

        let mut target = context.display.draw();
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
    });
}