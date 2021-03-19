mod renderer;
mod utility;
mod resources;

use glutin::{dpi::PhysicalSize, event::{DeviceEvent, ElementState::{Pressed, Released}, Event, KeyboardInput, VirtualKeyCode::{self, *}, WindowEvent}, event_loop::ControlFlow, window::Fullscreen};

use cgmath::{InnerSpace, Vector3};
use rand::Rng;
use std::{borrow::BorrowMut, env, path::Path, sync::{Arc, Mutex, RwLock}, thread};

use resources::Resources;
use renderer::{Material, camera::CameraBuilder, program::Program, shader::Shader, vao::{
        VertexArrayObject,
        VertexAttributePointer
    }, vbo::VertexBufferObject};

use utility::{Direction, chronos::Chronos};

// TODO: currently lots of opengl stuff. Move all of it into renderer module

fn main() {
    let res = Resources::from_relative_path(Path::new("assets")).unwrap();
    
    let el = glutin::event_loop::EventLoop::new();
    
    let physical_size = PhysicalSize::new(500, 300);

    let wb = {
        let mut wb  = glutin::window::WindowBuilder::new()
            .with_title("TDT4230 Raytracer")
            .with_resizable(false)
            .with_inner_size(physical_size)
            .with_always_on_top(true);

        let args: Vec<String> = env::args().collect();
        for arg in args.iter().skip(1) {
            match &arg[..] {
                "-f" | "-F" => {
                    wb = wb.with_maximized(true)
                    .with_fullscreen(Some(Fullscreen::Borderless(el.primary_monitor())));
                },
                "-h" => {
                    let h_command = "\n-h => 'display this information'";
                    let f_command = "\n-f | -F => 'fullscreen mode'"; // TODO: fov and mouse sense should be connected to this somehow
                    println!("Rendering toy code{}{}", h_command, f_command);
                    return;
                },
                c => eprintln!("Unknown command '{}'", c)
            }
        }

        wb
    };

    let cb = glutin::ContextBuilder::new().with_vsync(true);

    
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    if let Err(e) = windowed_context.window().set_cursor_grab(true) {
        panic!("Error grabbing mouse, e: {}", e);
    }
    windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);
    
    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        let sf = windowed_context.window().scale_factor();
        let screen_dimensions = windowed_context.window().inner_size().to_logical::<u32>(sf);


        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the renderin thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        unsafe {
            gl::Viewport(0, 0, screen_dimensions.width as i32, screen_dimensions.height as i32); // set viewport
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

        let mut camera = CameraBuilder::new(90.0, screen_dimensions.width as i32)
            .with_aspect_ratio(screen_dimensions.width as f32 / screen_dimensions.height as f32 )
            .with_origin(Vector3::<f32>::new(0.0, 0.0, 0.0))
            .with_viewport_height(2.0)
            .with_sample_per_pixel(4)
            .with_max_bounce(20)
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
                    // |Ir |
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
        let turn_rate: f32 = 0.05; // TODO: internal for camera
        loop {
            chronos.tick();
            
            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::W           => camera.translate(&mut raytrace_program, &Direction::Front.into_vector3(), chronos.delta_time()),
                        VirtualKeyCode::A           => camera.translate(&mut raytrace_program, &Direction::Left.into_vector3(),  chronos.delta_time()),
                        VirtualKeyCode::S           => camera.translate(&mut raytrace_program, &Direction::Back.into_vector3(),  chronos.delta_time()),
                        VirtualKeyCode::D           => camera.translate(&mut raytrace_program, &Direction::Rigth.into_vector3(), chronos.delta_time()),
                        VirtualKeyCode::Space       => camera.translate(&mut raytrace_program, &Direction::Up.into_vector3(),    chronos.delta_time()),
                        VirtualKeyCode::LControl    => camera.translate(&mut raytrace_program, &Direction::Down.into_vector3(),  chronos.delta_time()),
                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                const PRECISION: f32 = 0.0001;
                if delta.1.abs() > PRECISION {
                    let amount =  turn_rate * chronos.delta_time() as f32 * -delta.1;
                    camera.turn_pitch(&mut raytrace_program, amount);
                }
                if delta.0.abs() > PRECISION {
                    let amount = turn_rate * chronos.delta_time() as f32 * -delta.0;
                    camera.turn_yaw(&mut raytrace_program, amount);
                } 
                *delta = (0.0, 0.0);
            }

            // for event in event_pump.poll_iter() {
            //     match event {
            //         Event::Quit { .. } => break 'main,
            //         Event::KeyDown { keycode, .. } => match keycode {
            //             Some(k) => {
            //                 match k {
            //                   

            //                     _ => (),
            //                 }
            //             },
            //             _ => (),
            //         }
            //         _ => {}
            //     }
            // }
            
            hittable_vao.bind();
            dispatch_compute(&mut raytrace_program, screen_dimensions.width, screen_dimensions.height);
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

            context.swap_buffers().unwrap();
        }
    }); 

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    },
                    _ => { }
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            },
            _ => { }
        }
    });

    // texture delete ...
}