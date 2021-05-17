mod renderer;
mod utility;
mod resources;

use glutin::{ContextWrapper, GlProfile, NotCurrent, dpi::PhysicalSize, event::{DeviceEvent, ElementState::{self, Pressed, Released}, Event, KeyboardInput, VirtualKeyCode::{self, *}, WindowEvent}, event_loop::ControlFlow, window::{Fullscreen, Window}};

use cgmath::{Vector3};
use std::{env, ffi::c_void, path::Path, sync::{Arc, Mutex, RwLock}, thread};

use resources::Resources;
use renderer::{Material, camera::{CameraBuilder, CameraSettings}, compute_shader::ComputeShader, octree::{Octree}, program::Program, shader::Shader, vao::{
        VertexArrayObject,
        VertexAttributePointer
    }, vbo::VertexBufferObject};

use utility::{Direction, chronos::Chronos, ply_point_loader};


// TODO: currently lots of opengl stuff. Move all of it into renderer module

fn main() {
    let res = Resources::from_relative_path(Path::new("assets")).unwrap();
    
    let el = glutin::event_loop::EventLoop::new();
    
    let physical_size = PhysicalSize::new(1280, 720);

    let mut wb  = glutin::window::WindowBuilder::new()
        .with_title("TDT4230 Raytracer")
        .with_resizable(false)
        .with_inner_size(physical_size)
        .with_always_on_top(true);
        
    let mut chronos: Chronos = Default::default();

    let args: Vec<String> = env::args().collect();
    for arg in args.iter().skip(1) {
        match &arg[..] {
            "-c" => {
                chronos.display_fps = false
            }
            "-f" | "-F" => {
                wb = wb.with_maximized(true)
                    .with_fullscreen(Some(Fullscreen::Borderless(el.primary_monitor())));
            },
            "-h" => {
                // TODO: c should default to opt-in
                let c_command = "\n-c => 'turn off fps display in terminal'";
                let h_command = "\n-h => 'display this information'";
                let f_command = "\n-f | -F => 'fullscreen mode'"; 
                println!("Rendering toy code{}{}{}", h_command, f_command, c_command);
                return;
            },
            c => eprintln!("Unknown command '{}'", c)
        }
    }

    let cb = glutin::ContextBuilder::new()
        .with_gl_profile(GlProfile::Core).with_vsync(true);
    
    let windowed_context = cb.with_vsync(true).build_windowed(wb, &el).unwrap();
    {
        // This seems to fail at random on X11, so try a couple of times before failing
        const MAX_GRAB_ATTEMPTS: u32 = 20;
        'grab: for x in 0..MAX_GRAB_ATTEMPTS {
            match windowed_context.window().set_cursor_grab(true) {
                Ok(()) => break 'grab,
                Err(e) => {
                    if x == MAX_GRAB_ATTEMPTS - 1 { 
                        eprintln!("Error grabbing mouse, e: {}", e);
                    }
                }
            }
        }
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

    #[derive(Clone, Copy)]
    enum ClickEvent {
        None = 2,
        Left = 1,
        Right = 0
    }
    // Set up shared device event storage
    let arc_left_mouse_events = Arc::new(Mutex::new(ClickEvent::None));
    // Make a reference of this tuple to send to the render thread
    let arc_left_mouse = Arc::clone(&arc_left_mouse_events);

    let sf = windowed_context.window().scale_factor();
    
    let logical_dimensions = windowed_context.window().inner_size().to_logical::<i32>(sf);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the renderin thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const c_void);
            c
        };
        unsafe {
            gl::Viewport(0, 0, physical_size.width, physical_size.height); // set viewport
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
            
            VertexArrayObject::new::<f32>(vec![pos, uv], vertices.id(), gl::FLOAT)
        };
        
        let mut raytrace_program = {
            let shader = Shader::from_resources(&res, "shaders/raytracer.comp").unwrap();
            let program = Program::from_shaders(&[shader]).unwrap();
            ComputeShader::new(program).unwrap() // TODO: handle this
        }; 
        
        let camera_config_changed = Arc::new(Mutex::new(false));
        let camera_config_changed_render = Arc::clone(&camera_config_changed);
        let mut camera = {
            let mut builder = CameraBuilder::new(90.0, logical_dimensions.width as i32);
            builder.with_aspect_ratio(logical_dimensions.width as f32 / logical_dimensions.height as f32)
                .with_origin(Vector3::<f32>::new(0.0, -0.1, -0.3))
                .with_viewport_height(2.0); 

            if let Ok(bytes) = res.load_buffer("settings/camera.ron") {
                let settings: CameraSettings = ron::de::from_bytes(&bytes[0..]).unwrap();
                builder.with_sample_per_pixel(settings.samples_per_pixel)
                    .with_max_bounce(settings.max_bounce)
                    .with_turn_rate(settings.turn_rate)
                    .with_normal_speed(settings.normal_speed)
                    .with_sprint_speed(settings.sprint_speed);

                let watch_path = res.to_abs_path("settings");
                let _camera_watcher = thread::spawn(move || {
                    use std::sync::mpsc;
                    use std::time;
                    use notify::{Watcher, DebouncedEvent};
        
                    let (tx, rx) = mpsc::channel();
                    let mut watcher = notify::watcher(tx, time::Duration::from_secs_f32(1.0)).unwrap();
                    watcher.watch(watch_path, notify::RecursiveMode::Recursive).unwrap();
        
                    loop {
                        match rx.recv() {
                            Ok(event) => {
                                match event {
                                    DebouncedEvent::Write(p) => {
                                        if p.ends_with("camera.ron") {
                                            println!("Camera settings changed");
                                            if let Ok(mut v) = camera_config_changed.lock() {
                                                *v = true;
                                            }
                                        }
                                    }
                                    _ => (),
                                }
                            },
                            Err(e) => eprintln!("watch error: {:?}", e),
                        }
                    }
                });
            } 

            builder.build(&mut raytrace_program.program).unwrap()
        };

         

        // We only use this texture, so we bind it before render loop and forget about it.
        // This is somewhat bad practice, but in our case, the consequenses are non existent
        camera.render_texture.bind();

        // TODO: use this data
        // let content = match ply_point_loader::from_resources(&res, "models/3x3x3_point.ply") {
        //     Err(e) => {
        //         panic!("{}", e);
        //     },
        //     Ok(file) => file,
        // };

        let octree_update_program = {
            let shader = Shader::from_resources(&res, "shaders/octree_update.comp").unwrap();
            let program = Program::from_shaders(&[shader]).unwrap();
            ComputeShader::new(program).unwrap() // TODO: handle this
        }; 

        // TODO: vao might not be needed for shader storage buffer? read spec 
        //       and update code accordingly
        let octree = { 
            const PRE_ALLOCATED_CELLS: usize = 100000;
            let vao = {
                use renderer::octree::{EMPTY, PARENT, LEAF};
                let mut allocated_cells =  Vec::<u32>::with_capacity(PRE_ALLOCATED_CELLS * 8 * 2);
                allocated_cells.append(& mut vec![
                    // cell 0 (root)
                    1, PARENT,  1, EMPTY, 
                    1, EMPTY,   10, PARENT,
                    1, EMPTY,   1, EMPTY, 
                    1, EMPTY,   1, PARENT,
                    // cell 1
                    2, PARENT,  2, PARENT, 
                    2, PARENT,  2, PARENT,
                    2, PARENT,  2, PARENT, 
                    2, PARENT,  2, PARENT,
                    // cell 2
                    3, PARENT,  3, PARENT, 
                    3, PARENT,  3, PARENT,
                    3, PARENT,  3, PARENT, 
                    3, PARENT,  3, PARENT,
                    // cell 3
                    4, PARENT,  4, PARENT, 
                    4, PARENT,  4, PARENT,
                    4, PARENT,  4, PARENT, 
                    4, PARENT,  4, PARENT,
                    // cell 4
                    5, PARENT,  5, PARENT, 
                    5, PARENT,  5, PARENT, 
                    5, PARENT,  5, PARENT,
                    5, PARENT,  5, PARENT,
                    // cell 5
                    6, PARENT,  6, PARENT, 
                    6, PARENT,  6, PARENT,
                    6, PARENT,  6, PARENT, 
                    6, PARENT,  6, PARENT,
                    // cell 6
                    7, PARENT, 7, PARENT, 
                    7, PARENT, 7, PARENT,
                    7, PARENT, 7, PARENT, 
                    7, PARENT, 7, PARENT,
                    // cell 7
                    8, PARENT, 8, PARENT, 
                    8, PARENT, 8, PARENT,
                    8, PARENT, 8, PARENT, 
                    8, PARENT, 8, PARENT,
                    // cell 8
                    9, EMPTY, 9, PARENT, 
                    9, PARENT, 9, EMPTY,
                    9, EMPTY, 9, PARENT, 
                    9, PARENT, 9, PARENT,
                    // cell 9
                    0, LEAF, 2, LEAF, 
                    0, LEAF, 1, LEAF,
                    0, LEAF, 3, LEAF, 
                    0, LEAF, 0, LEAF,
                    // cell 10
                    11, PARENT,  11, PARENT, 
                    11, EMPTY,  11, EMPTY,
                    11, EMPTY,  11, EMPTY, 
                    11, PARENT,  11, PARENT,
                    // cell 11
                    12, PARENT,  12, PARENT, 
                    12, PARENT,  12, PARENT,
                    12, PARENT,  12, PARENT, 
                    12, PARENT,  12, PARENT,
                    // cell 12
                    13, PARENT,  13, PARENT, 
                    13, PARENT,  13, PARENT,
                    13, PARENT,  13, PARENT, 
                    13, PARENT,  13, PARENT,
                    // cell 13
                    14, PARENT,  14, PARENT, 
                    14, PARENT,  14, PARENT, 
                    14, PARENT,  14, PARENT,
                    14, PARENT,  14, PARENT,
                    // cell 14
                    15, PARENT,  15, PARENT, 
                    15, PARENT,  15, PARENT,
                    15, PARENT,  15, PARENT, 
                    15, PARENT,  15, PARENT,
                    // cell 15
                    16, PARENT, 16, PARENT, 
                    16, PARENT, 16, PARENT,
                    16, PARENT, 16, PARENT, 
                    16, PARENT, 16, PARENT,
                    // cell 16
                    17, PARENT, 17, PARENT, 
                    17, PARENT, 17, PARENT,
                    17, PARENT, 17, PARENT, 
                    17, PARENT, 17, PARENT,
                    // cell 17
                    18, PARENT, 18, PARENT, 
                    18, PARENT, 18, PARENT,
                    18, PARENT, 18, PARENT, 
                    18, PARENT, 18, PARENT,
                    // cell 18
                    7, LEAF, 2, LEAF, 
                    6, LEAF, 1, LEAF,
                    0, LEAF, 3, LEAF, 
                    0, LEAF, 5, LEAF,
                    // ------
                ]);

                // 8 nodes, 2 ints per node, 8 cells
                for _ in 8 * 2 * 10..PRE_ALLOCATED_CELLS {
                    allocated_cells.push(EMPTY);
                }
    
                let cells_vbo = VertexBufferObject::new::<u32>(
                    allocated_cells,
                    gl::ARRAY_BUFFER,
                    gl::DYNAMIC_DRAW
                );
    
                let cells_attrib = VertexAttributePointer {
                    location: 4,
                    size: 2,
                    offset: 0
                };
                unsafe { gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, cells_vbo.id()); } 
                VertexArrayObject::new::<u32>(vec![cells_attrib], cells_vbo.id(), gl::UNSIGNED_INT)
            };
        
            {
                let lambe = Material::Lambertian as u32;
                let diele = Material::Dielectric as u32;
                let metal = Material::Metal as u32;
                let mat_vbo = VertexBufferObject::new::<u32>(
                    vec![
                    // |Type  |Attrib |Albedo |
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
                vao.append_vbo::<u32>(vec![mat_attrib], mat_vbo.id(), gl::UNSIGNED_INT);
            }
            
            {
                let albedo_vbo = VertexBufferObject::new::<f32>(
                    vec![
                    // |Albedo        
                        0.1, 0.2, 0.5, 
                        0.8, 0.8, 0.0,
                        0.8, 0.8, 0.8,
                        0.8, 0.6, 0.2,
                        0.2, 0.4, 0.8,
                        0.4, 0.8, 0.2,
                        0.2, 0.2, 0.2,
                    ],
                    gl::ARRAY_BUFFER,
                    gl::STATIC_DRAW
                );
                let mat_attrib = VertexAttributePointer {
                    location: 5,
                    size: 3,
                    offset: 0
                };
                unsafe { gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, albedo_vbo.id()); } 
                vao.append_vbo::<f32>(vec![mat_attrib], albedo_vbo.id(), gl::FLOAT);
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
                    location: 6,
                    size: 1,
                    offset: 0
                };
                unsafe { gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, metal_vbo.id()); } 
                vao.append_vbo::<f32>(vec![metal_attrib], metal_vbo.id(), gl::FLOAT);
            }

            {
                let dielectric_vbo = VertexBufferObject::new::<f32>(
                    vec![
                    // |Ir |
                        1.2,
                    ],
                    gl::ARRAY_BUFFER,
                    gl::STATIC_DRAW
                );
                let dielectric_attrib = VertexAttributePointer {
                    location: 7,
                    size: 1,
                    offset: 0
                };
                unsafe { gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 4, dielectric_vbo.id()); } 
                vao.append_vbo::<f32>(vec![dielectric_attrib], dielectric_vbo.id(), gl::FLOAT);
            }

           
            let o = Octree::new(
                Vector3::new(-0.5, -0.5, -1.0), 
                1.0, 
                10, 
                PRE_ALLOCATED_CELLS as i32, 
                19,
                100, 
                vao
            ).unwrap();

            if let Err(e) = o.init_global_buffers() { 
                eprintln!("{}", e);
                return;
            }
            o
        };

        // TODO: remove 
        let mut delta = Vec::<f32>::with_capacity(100 * 4);
        for i in 0..100 {
            delta.push(0.01 * i as f32);
            delta.push(0.0);
            delta.push(0.0);
            delta.push(0.0);
            delta.push(0.0);
        }

        let click_cooldown = 0.05;
        let mut last_click_count = 0.0;
        let mut active_voxel = 0;
        let render_size = (camera.render_texture.width(), camera.render_texture.height(), camera.render_texture.depth());
        loop {
            chronos.tick();

            if let Ok(change) = camera_config_changed_render.lock() {
                if *change {
                    // TODO: really bad idea to do blocking io in render thread ...
                    if let Ok(bytes) = res.load_buffer("settings/camera.ron") {
                        match ron::de::from_bytes::<CameraSettings>(&bytes[0..]) {
                            Ok(settings) => camera.apply_settings(&mut raytrace_program.program, settings),
                            Err(_) => (),
                        }
                        
                    }
                }
            }

            // TODO: all these events should be a application specific enum to avoid all of these mutexes
            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                let mut l_shift_used = false;
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::W           => camera.translate(&mut raytrace_program.program, &Direction::Front.into_vector3(), chronos.delta_time()),
                        VirtualKeyCode::A           => camera.translate(&mut raytrace_program.program, &Direction::Left.into_vector3(),  chronos.delta_time()),
                        VirtualKeyCode::S           => camera.translate(&mut raytrace_program.program, &Direction::Back.into_vector3(),  chronos.delta_time()),
                        VirtualKeyCode::D           => camera.translate(&mut raytrace_program.program, &Direction::Rigth.into_vector3(), chronos.delta_time()),
                        VirtualKeyCode::Space       => camera.translate(&mut raytrace_program.program, &Direction::Up.into_vector3(),    chronos.delta_time()),
                        VirtualKeyCode::LControl    => camera.translate(&mut raytrace_program.program, &Direction::Down.into_vector3(),  chronos.delta_time()),
                        VirtualKeyCode::G           => octree.update_vbo(&delta, delta.len(), &octree_update_program),
                        VirtualKeyCode::Key1        => active_voxel = 0,
                        VirtualKeyCode::Key2        => active_voxel = 1,
                        VirtualKeyCode::Key3        => active_voxel = 2,
                        VirtualKeyCode::Key4        => active_voxel = 3,
                        VirtualKeyCode::Key5        => active_voxel = 4,
                        VirtualKeyCode::Key6        => active_voxel = 6,
                        VirtualKeyCode::Key7        => active_voxel = 7,
                        VirtualKeyCode::Key8        => active_voxel = 8,
                        VirtualKeyCode::Key9        => active_voxel = 9,
                        VirtualKeyCode::LShift      => {
                            camera.set_speed_to_sprint();
                            l_shift_used = true;
                        },
                        _ => { }
                    }
                }
                if !l_shift_used {
                    camera.set_speed_to_normal();
                }
            }

            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                const PRECISION: f32 = 0.0001;
                if delta.1.abs() > PRECISION {
                    let amount =  chronos.delta_time() as f32 * -delta.1;
                    camera.turn_pitch(&mut raytrace_program.program, amount);
                }
                if delta.0.abs() > PRECISION {
                    let amount = chronos.delta_time() as f32 * -delta.0;
                    camera.turn_yaw(&mut raytrace_program.program, amount);
                } 
                *delta = (0.0, 0.0);
            }

            if last_click_count >= click_cooldown {
                if let Ok(device_event) = arc_left_mouse.lock() {
                    let event = *device_event;
                    match event {
                        ClickEvent::Left | ClickEvent::Right => {
                            let mut spawn_point = camera.look_at_world_point(octree.block_distance() * 4.0);
                            if octree.point_inside(&spawn_point) {
                                last_click_count = 0.0;
                                spawn_point = spawn_point - octree.min_point();
                                // move spawn_point into a unit square
                                spawn_point /= octree.scale();
                                delta[0] = spawn_point.x.abs();
                                delta[1] = spawn_point.y.abs(); 
                                delta[1] = spawn_point.y.abs(); 
                                delta[1] = spawn_point.y.abs(); 
                                delta[2] = spawn_point.z.abs();
                                delta[3] = 2.0 * (event as i32 as f32);
                                delta[4] = active_voxel as f32;
                                octree.update_vbo(&delta, 5, &octree_update_program);
                            }
                        }
                        ClickEvent::None => (),
                    }
                } 
            } 
            last_click_count += chronos.delta_time();
            
            
            octree.vao.bind();
            raytrace_program.dispatch_compute(render_size.0 + 1, render_size.1 + 1, render_size.2);
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
    let mut window_focus = true;
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }
        

        if window_focus {
            match event {
                Event::WindowEvent { event: WindowEvent::Focused(f), .. } => {
                    window_focus = f;
                }
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
                Event::DeviceEvent { event: DeviceEvent::Button { button, state }, .. } => {
                    let mut event = ClickEvent::None;
                    if state == ElementState::Pressed {
                        if button == 1 {
                            event = ClickEvent::Left;
                        } else if button == 3 {
                            event = ClickEvent::Right;
                        }
                    }

                    if let Ok(mut device_event) = arc_left_mouse_events.lock() {
                        *device_event = event;
                    }
                }
                _ => { }
            }
        } else { // window not in focus
            match event {
                Event::WindowEvent { event: WindowEvent::Focused(f), .. } => {
                    window_focus = f;
                },
                _ => { }
            }
        }
    });
}