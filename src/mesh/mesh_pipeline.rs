use std::mem;

use super::geometry::Vertex;
use super::consts::DEPTH_FORMAT;
use super::material::MaterialFactorsUpload;

pub struct MeshPipeline {
    pub part_bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline: wgpu::RenderPipeline,
}

impl MeshPipeline {
    fn new(
        swapchain_format: wgpu::TextureFormat,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
        part_bind_group_layout: wgpu::BindGroupLayout,
        vs_module: wgpu::ShaderModule,
        fs_module: wgpu::ShaderModule,
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[
                global_bind_group_layout,
                mesh_bind_group_layout,
                &part_bind_group_layout,
            ],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x3, 1 => Float32x3, 2 => Float32x4, 3 => Float32x2,
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[
                    wgpu::ColorTargetState {
                        format: swapchain_format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    },
                    wgpu::ColorTargetState {
                        format: swapchain_format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    },
                ],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                clamp_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        MeshPipeline {
            part_bind_group_layout,
            pipeline,
        }
    }

    pub fn textured_unlit(
        swapchain_format: wgpu::TextureFormat,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let part_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Material factors
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false, filtering: true },
                        count: None,
                    },
                    // Base texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        // Get shaders
        let vs_module = device.create_shader_module(
            &wgpu::include_spirv!("shaders/tex_unlit_vert.spv")
        );
        let fs_module = device.create_shader_module(
            &wgpu::include_spirv!("shaders/tex_unlit_frag.spv")
        );

        MeshPipeline::new(
            swapchain_format,
            device,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }

    pub fn textured(
        swapchain_format: wgpu::TextureFormat,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let part_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Material factors
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false, filtering: true },
                        count: None,
                    },
                    // Base texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        // Get shaders
        let vs_module = device.create_shader_module(
            &wgpu::include_spirv!("shaders/pbr_vert.spv")
        );
        let fs_module = device.create_shader_module(
            &wgpu::include_spirv!("shaders/tex_pbr_frag.spv")
        );

        MeshPipeline::new(
            swapchain_format,
            device,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }

    pub fn textured_norm(
        swapchain_format: wgpu::TextureFormat,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let part_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Material factors
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false, filtering: true },
                        count: None,
                    },
                    // Base texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Normal map
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        // Get shaders
        let vs_module = device.create_shader_module(
            &wgpu::include_spirv!("shaders/pbr_vert.spv")
        );
        let fs_module = device.create_shader_module(
            &wgpu::include_spirv!("shaders/tex_norm_frag.spv")
        );

        MeshPipeline::new(
            swapchain_format,
            device,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }

    pub fn textured_norm_mat(
        swapchain_format: wgpu::TextureFormat,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let part_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Material factors
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false, filtering: true },
                        count: None,
                    },
                    // Base texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Normal map
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        // Get shaders
        let vs_module = device.create_shader_module(
            &wgpu::include_spirv!("shaders/pbr_vert.spv")
        );
        let fs_module = device.create_shader_module(
            &wgpu::include_spirv!("shaders/tex_norm_pbr_frag.spv")
        );

        MeshPipeline::new(
            swapchain_format,
            device,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }

    pub fn textured_emissive(
        swapchain_format: wgpu::TextureFormat,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let part_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Material factors
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false, filtering: true },
                        count: None,
                    },
                    // Base texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Normal map
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        // Get shaders
        let vs_module = device.create_shader_module(
            &wgpu::include_spirv!("shaders/pbr_vert.spv")
        );
        let fs_module = device.create_shader_module(
            &wgpu::include_spirv!("shaders/tex_emiss_pbr_frag.spv")
        );

        MeshPipeline::new(
            swapchain_format,
            device,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }

    pub fn untextured(
        swapchain_format: wgpu::TextureFormat,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let part_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Material factors
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                            ),
                        },
                        count: None,
                    },
                ],
            });

        // Get shaders
        let vs_module = device.create_shader_module(
            &wgpu::include_spirv!("shaders/pbr_vert.spv")
        );
        let fs_module = device.create_shader_module(
            &wgpu::include_spirv!("shaders/untex_pbr_frag.spv")
        );

        MeshPipeline::new(
            swapchain_format,
            device,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }
}

