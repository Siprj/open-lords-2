#[macro_use]
extern crate glium;
extern crate image;

use std::io::Cursor;

fn main() {
    use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter};
    #[allow(unused_imports)]
    use glium::{glutin, Surface};

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: (f32, f32, f32),
        tex_coords: (f32, f32),
    }

    implement_vertex!(Vertex, position, tex_coords);

    const X_DIM: u32 = 3;
    const Y_DIM: u32 = 3;

    let mut indices_raw_data: Vec<u32> = Vec::new();
    indices_raw_data.reserve((X_DIM as usize) * (Y_DIM as usize) * 6);
    let mut shape_raw_data = Vec::new();
    shape_raw_data.reserve((X_DIM as usize) * (Y_DIM as usize) * 4);
    for y in 0..Y_DIM {
        let y_offset = y as f32 * 15.0;
        for x in 0..X_DIM {
            let x_offset = x as f32 * 58.0 + if y % 2 == 0 { 0.0 } else { 29.0 };
            shape_raw_data.push(Vertex {position: (x_offset + 0.0, y_offset + 0.0, 1.0), tex_coords: (0.0, 1.0)});
            shape_raw_data.push(Vertex {position: (x_offset + 58.0, y_offset + 0.0, 1.0), tex_coords: (1.0, 1.0)});
            shape_raw_data.push(Vertex {position: (x_offset + 0.0, y_offset + 30.0, 0.0), tex_coords: (0.0, 0.0)});
            shape_raw_data.push(Vertex {position: (x_offset + 58.0, y_offset + 30.0, 0.0), tex_coords: (1.0, 0.0)});
            let indices_offset = (y * X_DIM + x) * 4;
            indices_raw_data.push(indices_offset + 0);
            indices_raw_data.push(indices_offset + 1);
            indices_raw_data.push(indices_offset + 2);
            indices_raw_data.push(indices_offset + 2);
            indices_raw_data.push(indices_offset + 1);
            indices_raw_data.push(indices_offset + 3);
        }
    }

    let shape = glium::vertex::VertexBuffer::new(
        &display,
        &shape_raw_data
    ).unwrap();

    let indices = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &indices_raw_data,
    )
    .unwrap();
//    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
//                                          &teapot::INDICES).unwrap();

    let image = image::load(
        Cursor::new(&include_bytes!("../assets/Roads1a.png")[..]),
        image::PNG,
    )
    .unwrap()
    .to_rgba();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let diffuse_texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

    let vertex_shader_src = r#"
        #version 150

        in vec3 position;
        in vec2 tex_coords;

        out vec2 v_tex_coords;

        uniform mat4 perspective;
        uniform mat4 model;
        uniform mat4 view;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = perspective * view * model * vec4(position, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;

        out vec4 color;

        uniform sampler2D diffuse_tex;

        void main() {
            color = texture(diffuse_tex, v_tex_coords).rgba;
        }
    "#;

    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

    event_loop.run(move |event, _, control_flow| {
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

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

        // let view = view_matrix(&[0.5, 0.2, -3.0], &[-0.5, -0.2, 3.0], &[0.0, 1.0, 0.0]);
        let view = [
            [10.0, 0.0, 0.0, 0.0],
            [0.0, 10.0, 0.0, 0.0],
            [0.0, 0.0, 10.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32],
        ];

        let mut target = display.draw();
        target.clear_color_and_depth((0.7, 0.7, 0.7, 1.0), 1.0);

        let model = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32],
        ];

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            blend: glium::Blend::alpha_blending(),
            ..Default::default()
        };
        let perspective = ortho_projection_matrix(0.0, 1920.0, 0.0, 1080.0, -100.0, 100.0);

        target
            .draw(
                &shape,
                &indices,
                &program,
                &uniform! { model: model, view: view, perspective: perspective,
                            diffuse_tex: diffuse_texture.sampled()
                                .minify_filter(MinifySamplerFilter::Nearest)
                                .magnify_filter(MagnifySamplerFilter::Nearest)
                },
                &params,
            )
            .unwrap();
        target.finish().unwrap();
    });
}

fn ortho_projection_matrix(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> [[f32; 4]; 4] {
    let m00 = 2.0 / (right - left);
    let m11 = -2.0 / (top - bottom);
    let m22 = -(2.0 / (far - near));
    [
        [m00, 0.0, 0.0, 0.0],
        [0.0, m11, 0.0, 0.0],
        [0.0, 0.0, m22, 0.0],
        [-1.0, 1.0, 0.0, 1.0],
    ]
}

fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [
        up[1] * f[2] - up[2] * f[1],
        up[2] * f[0] - up[0] * f[2],
        up[0] * f[1] - up[1] * f[0],
    ];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [
        f[1] * s_norm[2] - f[2] * s_norm[1],
        f[2] * s_norm[0] - f[0] * s_norm[2],
        f[0] * s_norm[1] - f[1] * s_norm[0],
    ];

    let p = [
        -position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
        -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
        -position[0] * f[0] - position[1] * f[1] - position[2] * f[2],
    ];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}
