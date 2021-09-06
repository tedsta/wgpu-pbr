use std::f32::consts::PI;

use ultraviolet::{Rotor3, Vec3};
use wgpu_pbr::{Camera, Renderer, Scene, PointLight};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    event::{self, WindowEvent, MouseScrollDelta},
};

fn main() {
    let event_loop = EventLoop::new();

    let title = "wgpu-pbr basic example";

    let window = winit::window::Window::new(&event_loop).unwrap();
    window.set_title(title);

    #[cfg(not(target_arch = "wasm32"))]
    {
        futures::executor::block_on(run_async(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run_async(event_loop, window));
    }
}


async fn run_async(event_loop: EventLoop<()>, window: winit::window::Window) {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

    let initial_screen_size = window.inner_size();
    let surface = unsafe { instance.create_surface(&window) };
    let needed_extensions = wgpu::Features::empty();

    let adapter = 
        instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();
    let adapter_features = adapter.features();

    let (device, queue) =
        adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: adapter_features & needed_extensions,
                limits: wgpu::Limits::default(),
            },
            None,
        ).await.unwrap();

    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        //format: wgpu::TextureFormat::Bgra8UnormSrgb,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: initial_screen_size.width,
        height: initial_screen_size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &surface_config);

    ////////////////////////////////////

    let camera = Camera::new(surface_config.width as f32 / surface_config.height as f32);
    let mut scene = Scene::new(camera);
    let mut renderer = Renderer::new(&surface_config, device, queue);

    #[cfg(not(target_arch = "wasm32"))]
    let mesh_id = {
        scene.add_mesh(renderer.mesh_from_file(
            "assets/models/SciFiHelmet.glb", true,
        ))
    };
    #[cfg(target_arch = "wasm32")]
    let mesh_id = {
        let scifi_helmet_bytes = include_bytes!("../assets/models/SciFiHelmet.glb");
        let mesh_parts = renderer.gltf_mesh_parts_from_reader(
            "assets/models/SciFiHelmet.glb",
            std::io::Cursor::new(scifi_helmet_bytes.as_ref()),
        );
        scene.add_mesh(renderer.mesh_from_parts(&mesh_parts))
    };
    // Unnecessary but perhaps educational?
    scene.mesh(mesh_id).position = Vec3::zero();
    scene.mesh(mesh_id).scale = Vec3::broadcast(1.0);

    // We'll position these lights down in the render loop
    let light0 = scene.add_point_light(PointLight {
        pos: [0.0; 3],
        color: [1.0, 0.3, 0.3],
        intensity: 800.0,
    });

    let light1 = scene.add_point_light(PointLight {
        pos: [0.0; 3],
        color: [0.3, 1.0, 0.3],
        intensity: 800.0,
    });

    let light2 = scene.add_point_light(PointLight {
        pos: [0.0; 3],
        color: [0.3, 0.3, 1.0],
        intensity: 800.0,
    });

    let winit::dpi::PhysicalSize { width: win_w, height: win_h } = window.inner_size();
    let win_center_x = win_w / 2;
    let win_center_y = win_h / 2;
    let _ignore_error = window
        .set_cursor_position(winit::dpi::LogicalPosition::new(win_center_x, win_center_y))
        .map_err(|_| eprintln!("unable to set cursor position"));
    window.set_maximized(true);

    let mut player_rot_x: f32 = 0.0;
    let mut player_rot_y: f32 = 0.0;
    let mut player_rot = Rotor3::identity();
    let mut camera_distance: f32 = 15.0;
    let mut prev_mouse_x: f64 = 0.0;
    let mut prev_mouse_y: f64 = 0.0;

    let mut timer = timer::Timer::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };
        match event {
            event::Event::MainEventsCleared => window.request_redraw(),
            event::Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                surface_config.width = size.width;
                surface_config.height = size.height;
                surface.configure(&renderer.device, &surface_config);

                scene.camera.resize(surface_config.width as f32 / surface_config.height as f32);
                renderer.mesh_pass.resize(&surface_config, &mut renderer.device);
            }
            event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Escape),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }

                WindowEvent::CursorMoved { position, .. } => {
                    let delta_x = position.x - prev_mouse_x;
                    let delta_y = position.y - prev_mouse_y;
                    prev_mouse_x = position.x;
                    prev_mouse_y = position.y;

                    player_rot_x -= (delta_y as f32) * 0.5;
                    player_rot_y += (delta_x as f32) * 0.5;

                    player_rot =
                        Rotor3::from_rotation_xz(f32::to_radians(player_rot_y)) *
                        Rotor3::from_rotation_yz(f32::to_radians(player_rot_x));
                }

                WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, y), .. } => {
                    camera_distance -= y * 0.5;
                }

                _ => { }
            }
            event::Event::RedrawRequested(_) => {
                let elapsed = timer.get_elapsed_micros();
                let elapsed_seconds = elapsed as f32 / 1_000_000.0;

                // Orbit them lights
                scene.point_light(light0).pos = [
                    10.0 * f32::cos(elapsed_seconds + 0.0 / 3.0 * 2.0 * PI),
                    10.0,
                    10.0 * f32::sin(elapsed_seconds + 0.0 / 3.0 * 2.0 * PI),
                ];
                scene.point_light(light1).pos = [
                    10.0 * f32::cos(elapsed_seconds + 1.0 / 3.0 * 2.0 * PI),
                    10.0,
                    10.0 * f32::sin(elapsed_seconds + 1.0 / 3.0 * 2.0 * PI),
                ];
                scene.point_light(light2).pos = [
                    10.0 * f32::cos(elapsed_seconds + 2.0 / 3.0 * 2.0 * PI),
                    10.0,
                    10.0 * f32::sin(elapsed_seconds + 2.0 / 3.0 * 2.0 * PI),
                ];

                // Update camera
                let mut cam_offset = Vec3::new(0.0, 0.0, -camera_distance);
                player_rot.rotate_vec(&mut cam_offset);
                scene.camera.look_at(
                    cam_offset,
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                );

                // Render scene
                let frame = surface.get_current_frame().expect("output frame");
                let mut encoder =
                    renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: None,
                    });
                renderer.render(&frame.output.texture.create_view(&wgpu::TextureViewDescriptor::default()), &mut encoder, &scene);
                renderer.queue.submit(Some(encoder.finish()));
            }
            _ => (),
        }
    });
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
mod timer {
    use std::time::Instant;

    pub struct Timer {
        last: Instant,
    }

    impl Timer {
        pub fn new() -> Timer {
            let now = Instant::now();
            Timer {
                last: now,
            }
        }

        pub fn get_elapsed_micros(&mut self) -> u64 {
            let now = Instant::now();
            let duration = now.duration_since(self.last);
            let interval = duration.as_micros() as u64;

            interval
        }

        pub fn clear(&mut self) {
            let now = Instant::now();
            self.last = now;
        }
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod timer {
    pub struct Timer {
        last: u64,
    }

    impl Timer {
        pub fn new() -> Timer {
            let now = web_sys::window().expect("should have a window in this context")
                .performance()
                .expect("performance should be available")
                .now() as u64;
            Timer {
                last: now,
            }
        }

        pub fn get_elapsed_micros(&mut self) -> u64 {
            let now = web_sys::window().expect("should have a window in this context")
                .performance()
                .expect("performance should be available")
                .now() as u64;

            let interval = now - self.last;

            interval * 1000 // Millis to micros
        }

        pub fn clear(&mut self) {
            let now = web_sys::window().expect("should have a window in this context")
                .performance()
                .expect("performance should be available")
                .now() as u64;
            self.last = now;
        }
    }
}

