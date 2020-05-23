extern crate gl;
extern crate sdl2;

mod renderer;
mod resources;

use sdl2::event::Event;
use resources::Resources;
use std::path::Path;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    {
        // Specify context version
        // currently we hardcode Opengl Core 4.5
        let gl_attr = video_subsystem.gl_attr();

        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 5);
    }

    let window_x: u32 = 900;
    let window_y: u32 = 700;

    let window = video_subsystem
        .window("GPUPE prototype", window_x, window_y)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    // keep context alive with varible
    let _gl_context = window.gl_create_context().unwrap();
    let _gl_load_with = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);
    let _gl_viewport_load_with = gl::Viewport::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

    unsafe {
        gl::Viewport(0, 0, window_x as i32, window_y as i32); // set viewport
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    let triangle_program = {
        use renderer::{shader::Shader, program::Program};

        let shaders = {
            use std::ffi::CString;

            let vert_shader = Shader::from_vert_source(
                &CString::new(include_str!("../resources/shaders/triangle.vert")).unwrap()
            ).unwrap();
        
            let frag_shader = Shader::from_frag_source(
                &CString::new(include_str!("../resources/shaders/triangle.frag")).unwrap()
            ).unwrap();

            &[vert_shader, frag_shader]
        };

        Program::from_shaders(shaders).unwrap()
    };

    let vertices: Vec<f32> = vec![
        -0.5, -0.5, 0.0,
        0.5, -0.5, 0.0,
        0.0, 0.5, 0.0
    ];

    let mut vbo: gl::types::GLuint = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
    }

    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, // target
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
            vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
            gl::STATIC_DRAW, // usage
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0); // unbind the buffer
    }

    let mut vao: gl::types::GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
    }

    unsafe {
        gl::BindVertexArray(vao);
        
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        gl::VertexAttribPointer(
            0, // index of the generic vertex attribute ("layout (location = 0)")
            3, // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            std::ptr::null() // offset of the first component
        );

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }



    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode, .. } => println!("Keydown: {:?}", keycode),
                Event::KeyUp { keycode, .. } => println!("Keyup: {:?}", keycode),
                _ => {}
            }
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        triangle_program.set_used();

        unsafe {
            gl::BindVertexArray(vao);
            gl::DrawArrays(
                gl::TRIANGLES, // mode
                0, // starting index in the enabled arrays
                3 // number of indices to be rendered
            );
        }

        window.gl_swap_window();
    }
}