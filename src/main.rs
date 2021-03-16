use cgmath::Vector3;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::{borrow::BorrowMut, path::Path};

mod renderer;
mod utility;
mod resources;

use resources::Resources;
use renderer::{camera::CameraBuilder, program::Program, shader::Shader, vao::{
        VertexArrayObject,
        VertexAttributePointer
    }, vbo::VertexBufferObject};

use utility::{
    input_handler::InputHandler,
    chronos::Chronos,
};

// TODO: currently lots of opengl stuff. Move all of it into renderer module

fn main() {
    let res = Resources::from_relative_path(Path::new("assets")).unwrap();
    
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    {
        // Specify context version
        // currently we hardcode Opengl Core 4.5
        let gl_attr = video_subsystem.gl_attr();

        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 5);
    }

    let window_x: u32 = 910;
    let window_y: u32 = 512;

    let window = video_subsystem
        .window("TDT4230 Raytracer", window_x, window_y)
        .opengl()
        // .resizable()
        .build()
        .unwrap();

    // keep context alive with variable
    let _gl_context = window.gl_create_context().unwrap();
    let _gl_load_with = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);
    let _gl_viewport_load_with = gl::Viewport::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

    unsafe {
        gl::Viewport(0, 0, window_x as i32, window_y as i32); // set viewport
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    // create quad data
    let quad_program = Program::from_resources(&res, "shaders/quad").unwrap();
    let quad_indices = VertexBufferObject::new::<u32>(
        vec![
            0, 1, 2,
            0, 1, 3
        ],
        gl::ELEMENT_ARRAY_BUFFER,
        gl::STATIC_DRAW
    );
    let quad_vao = { 
        let pos = VertexAttributePointer {
            location: 0,
            size: 3,
            offset: 0
        };

        let uv = VertexAttributePointer {
            location: 1,
            size: 2,
            offset: 3
        };

        let vertices = VertexBufferObject::new::<f32>(
            vec![
            //   x,    y,   z,   u,   v   
                -1.0, -1.0, 0.0, 0.0, 0.0, // bottom left
                 1.0,  1.0, 0.0, 1.0, 1.0, // top right
                -1.0,  1.0, 0.0, 0.0, 1.0, // top left
                 1.0, -1.0, 0.0, 1.0, 0.0  // bottom right
            ],
            gl::ARRAY_BUFFER,
            gl::STATIC_DRAW
        );

        VertexArrayObject::new(vec![pos, uv], 5, vertices.id())
    };

    // TODO: create a compute shader abstraction, used this in the abstraction somewhere where it can be shared
    // Retrieve work group count limit
    let mut work_group_count_limit = [0, 0, 0];
    unsafe {
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_COUNT, 0, &mut work_group_count_limit[0]);
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_COUNT, 1, &mut work_group_count_limit[1]);
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_COUNT, 2, &mut work_group_count_limit[2]);
    }
    let _work_group_count_limit = work_group_count_limit;

    // Retrieve work group size limit
    let mut work_group_size_limit = [0, 0, 0];
    unsafe {
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_SIZE, 0, &mut work_group_size_limit[0]);
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_SIZE, 1, &mut work_group_size_limit[1]);
        gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_SIZE, 2, &mut work_group_size_limit[2]);
    }
    let _work_group_size_limit = work_group_size_limit;

    let mut work_group_invocation_limit = 0;
    unsafe {
        gl::GetIntegerv(gl::MAX_COMPUTE_WORK_GROUP_INVOCATIONS, &mut work_group_invocation_limit);
    }
    let _work_group_invocation_limit = work_group_invocation_limit;

    let mut raytrace_program = {
        let shader = Shader::from_resources(&res, "shaders/raytracer.comp").unwrap();
        Program::from_shaders(&[shader]).unwrap()
    }; 

    fn dispatch_compute(program: &mut Program, window_x: u32, window_y: u32) {
        program.bind();

        unsafe {
            gl::DispatchCompute(window_x, window_y, 1);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        }

        Program::unbind();
    }

    let camera = CameraBuilder::new(window_x as i32)
        .with_aspect_ratio(window_x as f32 / window_y as f32 )
        .with_origin(Vector3::<f32>::new(0.0, 0.0, 0.0))
        .with_viewport_height(2.0)
        .with_sample_per_pixel(4) // the bigger resolution, the less we need of this
        .with_max_bounce(20)
        .build(&mut raytrace_program);

    // We only use this texture, so we bind it before render loop and forget about it.
    // This is somewhat bad practice, but in our case, the consequenses are non existent
    camera.render_texture.bind();

    // TODO: vao might not be needed for shader storage buffer? read spec 
    //       and update code accordingly
    let hittable_vao = { 
        let attrib = VertexAttributePointer {
            location: 0,
            size: 4,
            offset: 0
        };

        let sphere_vbo = VertexBufferObject::new::<f32>(
            vec![
            // |x      |y      |z      |radius|
                // -1.0,    0.0,    -1.1,   0.5,
                0.0,    0.0,    -1.0,   0.5,
                // 1.0,    0.0,    -1.1,   0.5,
                0.0,   -100.5,  -1.0,   100.0,
            ],
            gl::ARRAY_BUFFER,
            gl::DYNAMIC_DRAW
        );
        unsafe { gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, sphere_vbo.id()); } 

        VertexArrayObject::new(vec![attrib], 4, sphere_vbo.id())
    };

    {
        // TODO: n^3-tree for voxel data: 
        //       https://developer.nvidia.com/gpugems/gpugems2/part-v-image-oriented-computing/chapter-37-octree-textures-gpu?fbclid=IwAR057O64JgQK8kvI9Wil4NCnGWBG1ueNIoboYATwHhocpxzNIAKnBQBdkNE
        let mut max_3d_size: i32 = 0;  
        unsafe { gl::GetIntegerv(gl::MAX_3D_TEXTURE_SIZE, max_3d_size.borrow_mut() as *mut i32); }
    }


    let mut chronos: Chronos = Default::default();
    let mut input_handler: InputHandler = Default::default();

    // TODO: lock screen from being stretched
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        chronos.tick();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(_) => input_handler.on_key_down(keycode),
                    _ => (),
                }
                Event::KeyUp { keycode, .. } => input_handler.on_key_up(keycode),
                _ => {}
            }
        }

        // TODO: this should be done by some sort of observer like pattern, but this will work for now
        //       as soon as we need runtime config for keybindings this will be a problem
        for keycode in &input_handler.active_keys {
            match keycode {
                Keycode::W => (),
                Keycode::A => (),
                Keycode::S => (),
                Keycode::D => (),
                _ => ()
            }
        }
        hittable_vao.bind();
        dispatch_compute(&mut raytrace_program, window_x, window_y);
        VertexArrayObject::unbind();

        quad_program.bind();
        quad_vao.bind();
        quad_indices.bind();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawElements(
                gl::TRIANGLES, 
                quad_indices.length(), 
                gl::UNSIGNED_INT,
                std::ptr::null()
            );
        }

        quad_indices.unbind();
        VertexArrayObject::unbind();
        Program::unbind();

        window.gl_swap_window();
    }
    // texture delete ...
}