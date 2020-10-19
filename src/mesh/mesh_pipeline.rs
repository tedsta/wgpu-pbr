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
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
        part_bind_group_layout: wgpu::BindGroupLayout,
        vs_module: wgpu::ShaderModule,
        fs_module: wgpu::ShaderModule,
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[
                global_bind_group_layout,
                mesh_bind_group_layout,
                &part_bind_group_layout,
            ],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[
                wgpu::ColorStateDescriptor {
                    format: sc_desc.format,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                },
                wgpu::ColorStateDescriptor {
                    format: sc_desc.format,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                },
            ],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float4,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float4,
                            offset: 3 * 4,
                            shader_location: 1,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float4,
                            offset: 4 * 4 + 4 * 4,
                            shader_location: 2,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float2,
                            offset: 3 * 4 + 3 * 4 + 4 * 4,
                            shader_location: 3,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        MeshPipeline {
            part_bind_group_layout,
            pipeline,
        }
    }

    pub fn textured_unlit(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let part_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                bindings: &[
                    // Material factors
                    wgpu::BindGroupLayoutEntry::new(
                        0,
                        wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                            ),
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        1,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::Sampler { comparison: false },
                    ),
                    // Base texture
                    wgpu::BindGroupLayoutEntry::new(
                        2,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    ),
                ],
            });

        // Get shaders
        let vs_module = device.create_shader_module(
            wgpu::include_spirv!("shaders/tex_unlit_vert.spv")
        );
        let fs_module = device.create_shader_module(
            wgpu::include_spirv!("shaders/tex_unlit_frag.spv")
        );

        MeshPipeline::new(
            sc_desc,
            device,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }

    pub fn textured(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let part_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                bindings: &[
                    // Material factors
                    wgpu::BindGroupLayoutEntry::new(
                        0,
                        wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                            ),
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        1,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::Sampler { comparison: false },
                    ),
                    // Base texture
                    wgpu::BindGroupLayoutEntry::new(
                        2,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    ),
                ],
            });

        // Get shaders
        let vs_module = device.create_shader_module(
            wgpu::include_spirv!("shaders/pbr_vert.spv")
        );
        let fs_module = device.create_shader_module(
            wgpu::include_spirv!("shaders/tex_pbr_frag.spv")
        );

        MeshPipeline::new(
            sc_desc,
            device,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }

    pub fn textured_norm(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let part_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                bindings: &[
                    // Material factors
                    wgpu::BindGroupLayoutEntry::new(
                        0,
                        wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                            ),
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        1,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::Sampler { comparison: false },
                    ),
                    // Base texture
                    wgpu::BindGroupLayoutEntry::new(
                        2,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    ),
                    // Normal map
                    wgpu::BindGroupLayoutEntry::new(
                        3,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    ),
                ],
            });

        // Get shaders
        let vs_module = device.create_shader_module(
            wgpu::include_spirv!("shaders/pbr_vert.spv")
        );
        let fs_module = device.create_shader_module(
            wgpu::include_spirv!("shaders/tex_norm_frag.spv")
        );

        MeshPipeline::new(
            sc_desc,
            device,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }

    pub fn textured_norm_mat(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let part_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                bindings: &[
                    // Material factors
                    wgpu::BindGroupLayoutEntry::new(
                        0,
                        wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                            ),
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        1,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::Sampler { comparison: false },
                    ),
                    // Base texture
                    wgpu::BindGroupLayoutEntry::new(
                        2,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    ),
                    // Normal map
                    wgpu::BindGroupLayoutEntry::new(
                        3,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        4,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        5,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    ),
                ],
            });

        // Get shaders
        let vs_module = device.create_shader_module(
            wgpu::include_spirv!("shaders/pbr_vert.spv")
        );
        let fs_module = device.create_shader_module(
            wgpu::include_spirv!("shaders/tex_norm_pbr_frag.spv")
        );

        MeshPipeline::new(
            sc_desc,
            device,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }

    pub fn textured_emissive(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let part_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                bindings: &[
                    // Material factors
                    wgpu::BindGroupLayoutEntry::new(
                        0,
                        wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                            ),
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        1,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::Sampler { comparison: false },
                    ),
                    // Base texture
                    wgpu::BindGroupLayoutEntry::new(
                        2,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    ),
                    // Normal map
                    wgpu::BindGroupLayoutEntry::new(
                        3,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        4,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        5,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        6,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    ),
                ],
            });

        // Get shaders
        let vs_module = device.create_shader_module(
            wgpu::include_spirv!("shaders/pbr_vert.spv")
        );
        let fs_module = device.create_shader_module(
            wgpu::include_spirv!("shaders/tex_emiss_pbr_frag.spv")
        );

        MeshPipeline::new(
            sc_desc,
            device,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }

    pub fn untextured(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &mut wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let part_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                bindings: &[
                    // Material factors
                    wgpu::BindGroupLayoutEntry::new(
                        0,
                        wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                            ),
                        },
                    ),
                ],
            });

        // Get shaders
        let vs_module = device.create_shader_module(
            wgpu::include_spirv!("shaders/pbr_vert.spv")
        );
        let fs_module = device.create_shader_module(
            wgpu::include_spirv!("shaders/untex_pbr_frag.spv")
        );

        MeshPipeline::new(
            sc_desc,
            device,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }
}

