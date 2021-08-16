use crate::executor::{Command, FineDirection};
use crate::pipe::{OutBuffer, Pipe};
use glium::glutin::dpi::PhysicalPosition;
use glium::glutin::event::{ElementState, MouseButton, VirtualKeyCode};
use glium::index::NoIndices;
use glium::{glutin, Surface, VertexBuffer};
use glium::{glutin::dpi::LogicalSize, glutin::event_loop::EventLoop, Display};
use std::sync::mpsc::Receiver;

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

    let image = glium::texture::RawImage2d::from_raw_rgb(fractal.into_raw(), dimensions);

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

fn handle_keyboard(key: VirtualKeyCode) -> Option<Command> {
    match key {
        VirtualKeyCode::LBracket => Some(Command::ZoomOut),
        VirtualKeyCode::RBracket => Some(Command::ZoomIn),
        VirtualKeyCode::Minus => Some(Command::LessIterations),
        VirtualKeyCode::Equals => Some(Command::MoreIterations),
        VirtualKeyCode::Key0 => Some(Command::SetPOI(0)),
        VirtualKeyCode::Key1 => Some(Command::SetPOI(1)),
        VirtualKeyCode::Key2 => Some(Command::SetPOI(2)),
        VirtualKeyCode::Key3 => Some(Command::SetPOI(3)),
        VirtualKeyCode::Key4 => Some(Command::SetPOI(4)),
        VirtualKeyCode::Key5 => Some(Command::SetPOI(5)),
        VirtualKeyCode::Key6 => Some(Command::SetPOI(6)),
        VirtualKeyCode::Key7 => Some(Command::SetPOI(7)),
        VirtualKeyCode::Key8 => Some(Command::SetPOI(8)),
        VirtualKeyCode::Key9 => Some(Command::SetPOI(9)),
        VirtualKeyCode::Space => Some(Command::GetState),
        VirtualKeyCode::Up => Some(Command::FineTune(FineDirection::Up)),
        VirtualKeyCode::Down => Some(Command::FineTune(FineDirection::Down)),
        VirtualKeyCode::Left => Some(Command::FineTune(FineDirection::Left)),
        VirtualKeyCode::Right => Some(Command::FineTune(FineDirection::Right)),
        _ => None,
    }
}

pub fn run(pipe: Pipe) {
    let event_loop = EventLoop::new();

    let display = glium::Display::new(
        glutin::window::WindowBuilder::new().with_inner_size(LogicalSize::new(800, 800)),
        glutin::ContextBuilder::new().with_vsync(true),
        &event_loop,
    )
    .unwrap();

    let plane = create_plane(&display);

    let program = create_program(&display);

    let mut fps_count = 0;
    let mut fps_measure = std::time::Instant::now() + std::time::Duration::from_secs(1);

    let mut mouse_position = PhysicalPosition::new(0.0, 0.0);

    event_loop.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                glutin::event::WindowEvent::KeyboardInput {
                    input,
                    device_id: _,
                    is_synthetic: _,
                } => {
                    if input.state == ElementState::Released {
                        // println!("Got keyboard event! {:?}", input);
                        if let Some(key) = input.virtual_keycode {
                            if let Some(cmd) = handle_keyboard(key) {
                                pipe.cmd_send.send(cmd).unwrap();
                            }
                        }
                    }
                }
                glutin::event::WindowEvent::MouseInput {
                    device_id: _,
                    state,
                    button,
                    modifiers: _,
                } => {
                    if state == ElementState::Released {
                        if button == MouseButton::Left {
                            // println!(
                            //     "Mouse click! pos = {:?} {:?} {:?}",
                            //     mouse_position, state, button
                            // );

                            pipe.cmd_send
                                .send(Command::ChangeOrigin(mouse_position.x, mouse_position.y))
                                .unwrap();
                        }
                    }
                }
                glutin::event::WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    modifiers: _,
                } => {
                    mouse_position = position;
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

        let next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(50_000);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        if let Some(texture) = get_texture(&display, &pipe.img_rcv) {
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

            // println!("FPS {}", fps_count);
            fps_count = 0;
        } else {
            fps_count += 1;
        }
    });
}
