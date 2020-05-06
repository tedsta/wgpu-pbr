use std::f32::consts::PI;

use wgpu_pbr::{Camera, Renderer, Scene};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    event::{self, WindowEvent},
};

fn main() {
    let event_loop = EventLoop::new();

    let title = "wgpu-pbr basic example";

    let (window, initial_screen_size, surface) = {
        let window = winit::window::Window::new(&event_loop).unwrap();
        window.set_title(title);
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);
        (window, size, surface)
    };
    window.set_maximized(true);

    let adapter = futures::executor::block_on(
        wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
    ).unwrap();

    let (device, queue) = futures::executor::block_on(
        adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions { anisotropic_filtering: false },
            limits: wgpu::Limits::default(),
        })
    );

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: initial_screen_size.width,
        height: initial_screen_size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    ////////////////////////////////////

    let camera = Camera::new(sc_desc.width as f32 / sc_desc.height as f32);
    let mut scene = Scene::new(camera);
    let mut renderer = Renderer::new(&sc_desc, device, queue);

    let mesh_id = scene.add_mesh(renderer.mesh_from_file(
        "assets/models/SciFiHelmet/SciFiHelmet.gltf", true,
    ));
    // Unnecessary but perhaps educational?
    scene.meshes[mesh_id].position = cgmath::Point3::new(0.0, 0.0, 0.0);
    scene.meshes[mesh_id].scale = cgmath::Vector3::new(1.0, 1.0, 1.0);

    // We'll position these lights down in the render loop
    let light0 = scene.add_point_light();
    scene.point_lights[light0].color = [1.0, 0.3, 0.3];
    scene.point_lights[light0].intensity = 100.0;

    let light1 = scene.add_point_light();
    scene.point_lights[light1].color = [0.3, 1.0, 0.3];
    scene.point_lights[light1].intensity = 100.0;

    let light2 = scene.add_point_light();
    scene.point_lights[light2].color = [0.3, 0.3, 1.0];
    scene.point_lights[light2].intensity = 100.0;

    scene.camera.look_at(
        cgmath::Point3::new(2.0, 2.0, 5.0), // Position
        cgmath::Point3::new(0.0, 0.0, 0.0), // Target
        cgmath::Vector3::new(0.0, 1.0, 0.0), // Up
    );

    println!("Entering render loop...");
    let start_time = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };
        match event {
            event::Event::MainEventsCleared => window.request_redraw(),
            event::Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                println!("Resizing window to {:?}", size);
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = renderer.device.create_swap_chain(&surface, &sc_desc);

                scene.camera.resize(sc_desc.width as f32 / sc_desc.height as f32);
                renderer.mesh_pass.resize(&sc_desc, &mut renderer.device);
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
                _ => { }
            }
            event::Event::RedrawRequested(_) => {
                let elapsed = start_time.elapsed();
                let elapsed_seconds = (elapsed.as_micros() as f32) / 1000000.0;

                // Orbit them lights
                scene.point_lights[light0].pos = [
                    10.0 * f32::cos(elapsed_seconds + 0.0 / 3.0 * 2.0 * PI),
                    10.0,
                    10.0 * f32::sin(elapsed_seconds + 0.0 / 3.0 * 2.0 * PI),
                ];
                scene.point_lights[light1].pos = [
                    10.0 * f32::cos(elapsed_seconds + 1.0 / 3.0 * 2.0 * PI),
                    10.0,
                    10.0 * f32::sin(elapsed_seconds + 1.0 / 3.0 * 2.0 * PI),
                ];
                scene.point_lights[light2].pos = [
                    10.0 * f32::cos(elapsed_seconds + 2.0 / 3.0 * 2.0 * PI),
                    10.0,
                    10.0 * f32::sin(elapsed_seconds + 2.0 / 3.0 * 2.0 * PI),
                ];

                let frame = swap_chain.get_next_texture().expect("output frame");
                let mut encoder =
                    renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: None,
                    });

                renderer.render(&frame.view, &mut encoder, &scene);

                renderer.queue.submit(&[encoder.finish()]);
            }
            _ => (),
        }
    });
}

