#[macro_use]
extern crate serde_derive;

use cgmath::{Transform, SquareMatrix, InnerSpace};

use wgpu_glyph::{Section, GlyphBrushBuilder};

use wgpu_pbr::{Camera, Renderer, Scene, PointLightData, SpotLightData};

pub struct PlayerInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
}

impl PlayerInput {
    pub fn new() -> Self {
        PlayerInput {
            forward: false,
            backward: false,
            left: false,
            right: false,
        }
    }
}

fn main() {
    use winit::{
        event_loop::{ControlFlow, EventLoop},
        event::{self, WindowEvent, MouseScrollDelta},
    };

    let event_loop = EventLoop::new();
    println!("Initializing the window...");

    let title = "wgpu-pbr glTF viewer";

    let (window, mut screen_size, surface) = {
        let window = winit::window::Window::new(&event_loop).unwrap();
        window.set_title(title);
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);
        (window, size, surface)
    };

    window.set_cursor_grab(true).expect("grab mouse cursor");

    let adapter = futures::executor::block_on(
        wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        ),
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
        width: screen_size.width,
        height: screen_size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    ////////////////////////////////////

    let camera = Camera::new(sc_desc.width as f32 / sc_desc.height as f32);
    let mut scene = Scene::new(camera);
    let mut renderer = Renderer::new(&sc_desc, device, queue);

    let player_light_id = scene.add_spot_light();
    scene.spot_lights[player_light_id].pos = [0.0, 0.0, 0.0];
    scene.spot_lights[player_light_id].color = [1.0, 0.8, 0.5];
    scene.spot_lights[player_light_id].dir = [0.0, 0.0, 1.0];
    scene.spot_lights[player_light_id].angle = 0.9;
    scene.spot_lights[player_light_id].range = 20.0;
    scene.spot_lights[player_light_id].smoothness = 0.5;
    scene.spot_lights[player_light_id].intensity = 5.0;

    let scene_desc = SceneDescription::load("assets/scene.toml");
    scene_desc.build_scene(&mut renderer, &mut scene);

    let mut checkpoint = std::time::Instant::now();

    let font: &[u8] = include_bytes!("../assets/fonts/Inconsolata-Regular.ttf");
    let mut glyph_brush = GlyphBrushBuilder::using_font_bytes(font)
        .expect("GlyphBrushBuilder::using_font_bytes")
        .build(&mut renderer.device, sc_desc.format);

    // Prepare to run
    let mut input = PlayerInput::new();

    let winit::dpi::PhysicalSize { width: win_w, height: win_h } = window.inner_size();
    let win_center_x = win_w / 2;
    let win_center_y = win_h / 2;
    window.set_cursor_position(winit::dpi::LogicalPosition::new(
        win_center_x, win_center_y,
    )).expect("set cursor position");
    window.set_maximized(true);

    let mut viewport = cgmath::Vector4::new(0, 0, win_w as i32, win_h as i32);

    let mut player_pos = cgmath::Point3::new(0.0, 0.0, 0.0);
    let mut player_rot_x: f32 = 0.0;
    let mut player_rot_y: f32 = 0.0;
    let mut player_rot = cgmath::Matrix4::identity();
    let mut camera_distance: f32 = 15.0;

    println!("Entering render loop...");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };
        match event {
            event::Event::MainEventsCleared => window.request_redraw(),
            event::Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                screen_size = size; 
                println!("Resizing window to {:?}", size);
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = renderer.device.create_swap_chain(&surface, &sc_desc);

                scene.camera.resize(sc_desc.width as f32 / sc_desc.height as f32);
                renderer.mesh_pass.resize(&sc_desc, &mut renderer.device);

                viewport = cgmath::Vector4::new(0, 0, size.width as i32, size.height as i32);
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
                    let winit::dpi::PhysicalSize { width: win_w, height: win_h } =
                        window.inner_size();
                    let win_center_x = (win_w / 2) as i32;
                    let win_center_y = (win_h / 2) as i32;
                    window.set_cursor_position(winit::dpi::LogicalPosition::new(
                        win_center_x, win_center_y,
                    )).expect("set cursor position");

                    let delta_x = -(position.x - win_center_x as f64);
                    let delta_y = position.y - win_center_y as f64;

                    player_rot_x += (delta_y as f32) * 0.02;
                    player_rot_y += (delta_x as f32) * 0.02;

                    player_rot =
                        cgmath::Matrix4::from_angle_y(cgmath::Deg(player_rot_y)) *
                        cgmath::Matrix4::from_angle_x(cgmath::Deg(player_rot_x));
                }

                WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, y), .. } => {
                    camera_distance -= y * 0.5;
                }

                WindowEvent::KeyboardInput {
                    input: event::KeyboardInput {
                        virtual_keycode: Some(virtual_keycode),
                        state: event::ElementState::Pressed,
                        ..
                    },
                    ..
                } => {
                    match virtual_keycode {
                        event::VirtualKeyCode::W => { input.forward = true; }
                        event::VirtualKeyCode::S => { input.backward = true; }
                        event::VirtualKeyCode::A => { input.left = true; }
                        event::VirtualKeyCode::D => { input.right = true; }
                        _ => { }
                    }
                }

                WindowEvent::KeyboardInput {
                    input: event::KeyboardInput {
                        virtual_keycode: Some(virtual_keycode),
                        state: event::ElementState::Released,
                        ..
                    },
                    ..
                } => {
                    match virtual_keycode {
                        event::VirtualKeyCode::W => { input.forward = false; }
                        event::VirtualKeyCode::S => { input.backward = false; }
                        event::VirtualKeyCode::A => { input.left = false; }
                        event::VirtualKeyCode::D => { input.right = false; }
                        _ => { }
                    }
                }

                _ => { }
            }
            event::Event::RedrawRequested(_) => {
                let elapsed = checkpoint.elapsed();
                checkpoint += elapsed;
                let dt = (elapsed.as_micros() as f32) / 1000000.0;

                {
                    let player_forward = player_rot.transform_vector(
                        cgmath::Vector3::new(0.0, 0.0, 1.0)
                    );
                    let player_strafe = player_rot.transform_vector(
                        cgmath::Vector3::new(1.0, 0.0, 0.0)
                    );

                    let mut move_vec = cgmath::Vector3::new(0.0, 0.0, 0.0);
                    if input.forward {
                        move_vec += player_forward * 1.0 * dt;
                    } else if input.backward {
                        move_vec += player_forward * -1.0 * dt;
                    }
                    if input.left {
                        move_vec += player_strafe * 1.0 * dt;
                    } else if input.right {
                        move_vec += player_strafe * -1.0 * dt;
                    }

                    if move_vec.magnitude() > 0.001 {
                        player_pos += move_vec.normalize() * 20.0 * dt;
                    }
                }

                let cam_offset = player_rot.transform_vector(
                    cgmath::Vector3::new(0.0, 0.0, -camera_distance)
                );

                let player_light_dir = player_rot.transform_vector(
                    cgmath::Vector3::new(0.0, 0.0, 1.0)
                );
                scene.spot_lights[player_light_id].pos = [
                    player_pos.x + cam_offset.x,
                    player_pos.y + cam_offset.y,
                    player_pos.z + cam_offset.z,
                ];
                scene.spot_lights[player_light_id].dir = [
                    player_light_dir.x,
                    player_light_dir.y,
                    player_light_dir.z,
                ];

                let cam_up = player_rot.transform_vector(
                    cgmath::Vector3::new(0.0, 1.0, 0.0)
                );
                scene.camera.look_at(
                    player_pos + cam_offset,
                    player_pos + player_light_dir * 1.0,
                    cam_up,
                );

                let frame = swap_chain.get_next_texture().expect("output frame");
                let mut encoder =
                    renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: None,
                    });

                renderer.render(&frame.view, &mut encoder, &scene);

                glyph_brush.queue(Section {
                    text: &format!(
                        "FPS: {}\nx: {}\ny: {}\nz: {}",
                        1.0 / dt,
                        player_pos.x,
                        player_pos.y,
                        player_pos.z,
                    ),
                    screen_position: (30.0, 30.0),
                    color: [1.0, 1.0, 1.0, 1.0],
                    scale: wgpu_glyph::Scale { x: 40.0, y: 40.0 },
                    bounds: (screen_size.width as f32, screen_size.height as f32),
                    ..Section::default()
                });
                glyph_brush.draw_queued(
                    &mut renderer.device,
                    &mut encoder,
                    &frame.view,
                    screen_size.width,
                    screen_size.height,
                ).expect("glyph draw queued");

                renderer.queue.submit(&[encoder.finish()]);
            }
            _ => (),
        }
    });
}

#[derive(Clone, Deserialize)]
pub struct PointLightDescription {
    red: f32,
    green: f32,
    blue: f32,
    intensity: f32,
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Deserialize)]
pub struct SpotLightDescription {
    red: f32,
    green: f32,
    blue: f32,
    intensity: f32,
    x: f32,
    y: f32,
    z: f32,
    dir_x: f32,
    dir_y: f32,
    dir_z: f32,
    range: f32,
    angle: f32,
    smoothness: f32,
}

#[derive(Clone, Deserialize)]
pub struct MeshDescription {
    mesh: String,
    lighting: Option<bool>,
    x: f32,
    y: f32,
    z: f32,
    rot_x: Option<f32>,
    rot_y: Option<f32>,
    rot_z: Option<f32>,
    scale_x: Option<f32>,
    scale_y: Option<f32>,
    scale_z: Option<f32>,
}

#[derive(Clone, Deserialize)]
pub struct SceneDescription {
    point_light: Option<Vec<PointLightDescription>>,
    spot_light: Option<Vec<SpotLightDescription>>,
    mesh: Option<Vec<MeshDescription>>,
}

impl SceneDescription {
    pub fn load(path: &str) -> Self {
        use std::io::Read;

        // Open and read config file
        let mut toml_str = String::new();
        std::fs::File::open(path)
            .and_then(|mut f| f.read_to_string(&mut toml_str))
            .expect("SceneDescription::load file");
        // Parse config file
        toml::from_str(&toml_str)
            .expect("SceneDescription::load toml")
    }

    pub fn build_scene(&self, renderer: &mut Renderer, scene: &mut Scene) {
        if let Some(ref point_lights) = self.point_light {
            for point_light in point_lights {
                scene.point_lights.insert(PointLightData::new(
                    [point_light.x, point_light.y, point_light.z],
                    point_light.intensity,
                    [point_light.red, point_light.green, point_light.blue],
                ));
            }
        }

        if let Some(ref spot_lights) = self.spot_light {
            for spot_light in spot_lights {
                let mut l = SpotLightData::zero();
                l.pos = [spot_light.x, spot_light.y, spot_light.z];
                l.color = [spot_light.red, spot_light.green, spot_light.blue];
                l.dir = [spot_light.dir_x, spot_light.dir_y, spot_light.dir_z];
                l.angle = spot_light.angle;
                l.range = spot_light.range;
                l.smoothness = spot_light.smoothness;
                l.intensity = spot_light.intensity;
                scene.spot_lights.insert(l);
            }
        }

        if let Some(ref meshes) = self.mesh {
            for mesh in meshes {
                let lighting = mesh.lighting.unwrap_or(true);
                let mesh_id = scene.add_mesh(renderer.mesh_from_file(
                    &mesh.mesh,
                    lighting
                ));
                scene.meshes[mesh_id].position = cgmath::Point3::new(
                    mesh.x,
                    mesh.y,
                    mesh.z,
                );
                scene.meshes[mesh_id].rotation =
                    cgmath::Matrix4::from_angle_y(cgmath::Deg(mesh.rot_y.unwrap_or(0.0))) *
                    cgmath::Matrix4::from_angle_x(cgmath::Deg(mesh.rot_x.unwrap_or(0.0))) *
                    cgmath::Matrix4::from_angle_z(cgmath::Deg(mesh.rot_z.unwrap_or(0.0)));
                scene.meshes[mesh_id].scale = cgmath::Vector3::new(
                    mesh.scale_x.unwrap_or(1.0),
                    mesh.scale_y.unwrap_or(1.0),
                    mesh.scale_z.unwrap_or(1.0),
                );
            }
        }
    }
}

