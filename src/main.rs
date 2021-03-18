use cgmath::{InnerSpace, Vector3};
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::{borrow::BorrowMut, path::Path};

mod renderer;
mod utility;
mod resources;

use resources::Resources;
use renderer::{Material, camera::CameraBuilder, program::Program, shader::Shader, vao::{
        VertexArrayObject,
        VertexAttributePointer
    }, vbo::VertexBufferObject};

use utility::{Direction, chronos::Chronos};

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

    let window_x: u32 = 500;
    let window_y: u32 = 300;

    let window = video_subsystem
        .window("TDT4230 Raytracer", window_x, window_y)
        .opengl()
        // .fullscreen()
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

        VertexArrayObject::new(vec![pos, uv], vertices.id())
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

    let mut camera = CameraBuilder::new(90.0, window_x as i32)
        .with_aspect_ratio(window_x as f32 / window_y as f32 )
        .with_view_dir(Vector3::<f32>::new(0.0, 0.0, -1.0))
        .with_origin(Vector3::<f32>::new(0.0, 0.0, 0.0))
        .with_viewport_height(2.0)
        .with_sample_per_pixel(4)
        .with_max_bounce(8)
        .build(&mut raytrace_program);

    // We only use this texture, so we bind it before render loop and forget about it.
    // This is somewhat bad practice, but in our case, the consequenses are non existent
    camera.render_texture.bind();

    // TODO: vao might not be needed for shader storage buffer? read spec 
    //       and update code accordingly
    let hittable_vao = { 
        let vao = {
            let mut default_spheres = vec![
                // |Position          |Radius  |Mat index|Padding |
                    0.0, -100.5, -1.0,  100.0,  1.0,      0.0, 0.0, 0.0, // big sphere
                    0.0,  0.0,   -1.0,  0.5,    0.0,      0.0, 0.0, 0.0, // middle sphere
                    4.0,  0.0,   -1.0,  0.5,    3.0,      0.0, 0.0, 0.0, // right sphere
                   -4.0,  0.0,   -1.0,  0.5,    2.0,      0.0, 0.0, 0.0, // hollow glass outer 
                   -4.0,  0.0,   -1.0, -0.4,    2.0,      0.0, 0.0, 0.0, // hollow glass inner
                    0.0,  0.0,    2.0,  0.5,    2.0,      0.0, 0.0, 0.0, // glass 
            ];

            let mut all_spheres = Vec::<f32>::with_capacity(default_spheres.len() + 11 * 11);
            all_spheres.append(&mut default_spheres);
            let mut rng = rand::thread_rng();
            let extends = Vector3::<f32>::new(4.0, 0.2, 0.0);
            for i in 0..11 {
                for j in 0..11 {
                    let center = Vector3::<f32>::new(i as f32 + 0.9 * rng.gen::<f32>(), 0.2, j as f32 + 0.9 * rng.gen::<f32>());
                    
                    if (center - extends).magnitude() > 0.9{
                        all_spheres.push(center.x);
                        all_spheres.push(center.y);
                        all_spheres.push(center.z);
                        all_spheres.push(0.2);

                        let mat: u32 = rng.gen_range(0..12);
                        all_spheres.push(mat as f32);
                        all_spheres.push(0.0);
                        all_spheres.push(0.0);
                        all_spheres.push(0.0);
                    }
                }
            }

            let sphere_vbo = VertexBufferObject::new::<f32>(
                all_spheres,
                gl::ARRAY_BUFFER,
                gl::STATIC_DRAW
            );
            let sphere_attrib = VertexAttributePointer {
                location: 0,
                size: 8,
                offset: 0
            };
            unsafe { gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, sphere_vbo.id()); } 
            VertexArrayObject::new(vec![sphere_attrib], sphere_vbo.id()) 
        };
       
        {
            let lambe = Material::Lambertian as u32;
            let diele = Material::Dielectric as u32;
            let metal = Material::Metal as u32;
            let mat_vbo = VertexBufferObject::new::<u32>(
                vec![
                // |Type  |Attrib |Albedo index|
                    lambe,  0,      0,
                    lambe,  0,      1, 
                    diele,  0,      2,  
                    metal,  0,      3,
                    metal,  1,      4,
                    metal,  2,      5,
                    metal,  3,      6,
                    diele,  0,      4,  
                    diele,  0,      5, 
                    lambe,  0,      6, 
                    lambe,  0,      5,
                    lambe,  0,      4,
                    lambe,  0,      3, 
                ],
                gl::ARRAY_BUFFER,
                gl::STATIC_DRAW
            );
            unsafe { gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, mat_vbo.id()); } 
            let mat_attrib = VertexAttributePointer {
                location: 0,
                size: 3,
                offset: 0
            };
            vao.append_vbo(vec![mat_attrib], mat_vbo.id());
        }
        
        {
            let albedo_vbo = VertexBufferObject::new::<f32>(
                vec![
                // |Albedo        |Padding|
                    0.1, 0.2, 0.5, 0.0, 
                    0.8, 0.8, 0.0, 0.0,
                    0.8, 0.8, 0.8, 0.0,
                    0.8, 0.6, 0.2, 0.0,
                    0.2, 0.4, 0.8, 0.0,
                    0.4, 0.8, 0.2, 0.0,
                    0.2, 0.2, 0.2, 0.0,
                ],
                gl::ARRAY_BUFFER,
                gl::STATIC_DRAW
            );
            let mat_attrib = VertexAttributePointer {
                location: 0,
                size: 4,
                offset: 0
            };
            unsafe { gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, albedo_vbo.id()); } 
            vao.append_vbo(vec![mat_attrib], albedo_vbo.id());
        }

        {
            let metal_vbo = VertexBufferObject::new::<f32>(
                vec![
                // |Fuzz |
                    0.1,
                    0.3,
                    0.4,
                    0.8,
                ],
                gl::ARRAY_BUFFER,
                gl::STATIC_DRAW
            );
            let metal_attrib = VertexAttributePointer {
                location: 0,
                size: 1,
                offset: 0
            };
            unsafe { gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, metal_vbo.id()); } 
            vao.append_vbo(vec![metal_attrib], metal_vbo.id());
        }

        {
            let dielectric_vbo = VertexBufferObject::new::<f32>(
                vec![
                // |Fuzz |
                    1.5,
                ],
                gl::ARRAY_BUFFER,
                gl::STATIC_DRAW
            );
            let dielectric_attrib = VertexAttributePointer {
                location: 0,
                size: 1,
                offset: 0
            };
            unsafe { gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 4, dielectric_vbo.id()); } 
            vao.append_vbo(vec![dielectric_attrib], dielectric_vbo.id());
        }

        vao
    };

    {
        // TODO: n^3-tree for voxel data: 
        //       https://developer.nvidia.com/gpugems/gpugems2/part-v-image-oriented-computing/chapter-37-octree-textures-gpu?fbclid=IwAR057O64JgQK8kvI9Wil4NCnGWBG1ueNIoboYATwHhocpxzNIAKnBQBdkNE
        let mut max_3d_size: i32 = 0;  
        unsafe { gl::GetIntegerv(gl::MAX_3D_TEXTURE_SIZE, max_3d_size.borrow_mut() as *mut i32); }
    }


    let mut chronos: Chronos = Default::default();
    // TODO: lock screen from being stretched
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        chronos.tick();

        // TODO: move to seperate thread
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(k) => {
                        match k {
                            Keycode::W => camera.translate(&mut raytrace_program, &Direction::Front.into_vector3(), chronos.delta_time()),
                            Keycode::A => camera.translate(&mut raytrace_program, &Direction::Left.into_vector3(),  chronos.delta_time()),
                            Keycode::S => camera.translate(&mut raytrace_program, &Direction::Back.into_vector3(),  chronos.delta_time()),
                            Keycode::D => camera.translate(&mut raytrace_program, &Direction::Rigth.into_vector3(), chronos.delta_time()),
                            Keycode::Space => camera.translate(&mut raytrace_program, &Direction::Up.into_vector3(), chronos.delta_time()),
                            Keycode::LCtrl => camera.translate(&mut raytrace_program, &Direction::Down.into_vector3(), chronos.delta_time()),
                            Keycode::Up => camera.turn_pitch(&mut raytrace_program, -2.0 * chronos.delta_time() as f32),
                            Keycode::Down => camera.turn_pitch(&mut raytrace_program, 2.0 * chronos.delta_time() as f32),
                            Keycode::Left => camera.turn_yaw(&mut raytrace_program, 2.0 * chronos.delta_time() as f32),
                            Keycode::Right => camera.turn_yaw(&mut raytrace_program, -2.0 * chronos.delta_time() as f32),
                            Keycode::Escape => break 'main,

                            _ => (),
                        }
                    },
                    _ => (),
                }
                _ => {}
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