use crate::*;
use std::collections::HashMap;

impl IntoInstance for PolygonMesh {
    type Instance = PolygonInstance;
    fn into_instance(&self, device: &Device, _: &Queue, desc: InstanceDescriptor) -> Self::Instance {
        let (vb, ib) = ExpandedPolygon::from(self).buffers(device);
        PolygonInstance {
            polygon: (Arc::new(vb), Arc::new(ib)),
            desc,
            id: None,
        }
    }
}

impl IntoInstance for StructuredMesh {
    type Instance = PolygonInstance;
    fn into_instance(&self, device: &Device, _: &Queue, desc: InstanceDescriptor) -> Self::Instance {
        let (vb, ib) = ExpandedPolygon::from(self).buffers(device);
        PolygonInstance {
            polygon: (Arc::new(vb), Arc::new(ib)),
            desc,
            id: None,
        }
    }
}

impl Default for Material {
    #[inline(always)]
    fn default() -> Material {
        Material {
            albedo: Vector4::new(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0, 1.0),
            roughness: 0.5,
            reflectance: 0.5,
        }
    }
}

impl Default for InstanceDescriptor {
    #[inline(always)]
    fn default() -> InstanceDescriptor {
        InstanceDescriptor {
            matrix: Matrix4::identity(),
            material: Default::default(),
            texture: None,
            backface_culling: true,
        }
    }
}

impl Material {
    pub fn buffer(&self, device: &Device) -> BufferHandler {
        let material_data: [f32; 6] = [
            self.albedo[0] as f32,
            self.albedo[1] as f32,
            self.albedo[2] as f32,
            self.albedo[3] as f32,
            self.roughness as f32,
            self.reflectance as f32,
        ];
        BufferHandler::new(
            device.create_buffer_init(&BufferInitDescriptor {
                contents: bytemuck::cast_slice(&material_data),
                usage: BufferUsage::UNIFORM,
                label: None,
            }),
            std::mem::size_of::<[f32; 6]>() as u64,
        )
    }
}

impl PolygonInstance {
    pub fn pipeline_with_shader(
        &self,
        vertex_shader: ShaderModuleSource,
        fragment_shader: ShaderModuleSource,
        scene: &DeviceHandler,
        layout: &PipelineLayout,
    ) -> Arc<RenderPipeline>
    {
        let device = scene.device();
        let sc_desc = scene.sc_desc();
        let cull_mode = if self.desc.backface_culling {
            CullMode::Back
        } else {
            CullMode::None
        };
        let vertex_module = device.create_shader_module(vertex_shader);
        let fragment_module = device.create_shader_module(fragment_shader);
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: Some(layout),
            vertex_stage: ProgrammableStageDescriptor {
                module: &vertex_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fragment_module,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: Some(DepthStencilStateDescriptor {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: StencilStateDescriptor {
                    front: wgpu::StencilStateFaceDescriptor::IGNORE,
                    back: wgpu::StencilStateFaceDescriptor::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
            }),
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint32,
                vertex_buffers: &[VertexBufferDescriptor {
                    stride: std::mem::size_of::<AttrVertex>() as BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &[
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float2,
                            offset: 3 * 4,
                            shader_location: 1,
                        },
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float3,
                            offset: 2 * 4 + 3 * 4,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
            label: None,
        });
        Arc::new(pipeline)
    }

    fn non_textured_bdl(&self, device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                // matrix
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // color
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: None,
        })
    }

    fn textured_bdl(&self, device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                // matrix
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // color
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // texture view
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Uint,
                        multisampled: false,
                    },
                    count: None,
                },
                // sampler
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler { comparison: false },
                    count: None,
                },
            ],
            label: None,
        })
    }

    pub fn matrix_buffer(&self, device: &Device) -> BufferHandler {
        let matrix_data: [[f32; 4]; 4] = self.desc.matrix.cast::<f32>().unwrap().into();
        BufferHandler::new(
            device.create_buffer_init(&BufferInitDescriptor {
                contents: bytemuck::cast_slice(&matrix_data),
                usage: BufferUsage::UNIFORM,
                label: None,
            }),
            std::mem::size_of::<[[f32; 4]; 4]>() as u64,
        )
    }

    pub fn textureview_and_sampler(
        &self,
        device: &Device,
        queue: &Queue,
    ) -> (TextureView, Sampler)
    {
        let texture_image = self.desc.texture.as_ref().unwrap();
        let rgba = texture_image.to_rgba8();
        let dim = texture_image.dimensions();
        let size = Extent3d {
            width: dim.0,
            height: dim.1,
            depth: 1,
        };
        let texture = device.create_texture(&TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            label: None,
        });
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            contents: &rgba,
            usage: BufferUsage::COPY_SRC,
            label: None,
        });
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_texture(
            BufferCopyView {
                buffer: &buffer,
                layout: TextureDataLayout {
                    offset: 0,
                    bytes_per_row: 4 * dim.0,
                    rows_per_image: dim.1,
                },
            },
            TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            size,
        );
        queue.submit(vec![encoder.finish()]);

        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
            label: None,
        });
        (view, sampler)
    }

    fn non_textured_bg(&self, device: &Device, layout: &BindGroupLayout) -> BindGroup {
        crate::create_bind_group(
            device,
            layout,
            vec![
                self.matrix_buffer(device).binding_resource(),
                self.desc.material.buffer(device).binding_resource(),
            ],
        )
    }
    fn textured_bg(&self, device: &Device, queue: &Queue, layout: &BindGroupLayout) -> BindGroup {
        let (view, sampler) = self.textureview_and_sampler(device, queue);
        crate::create_bind_group(
            device,
            layout,
            vec![
                self.matrix_buffer(device).binding_resource(),
                self.desc.material.buffer(device).binding_resource(),
                BindingResource::TextureView(&view),
                BindingResource::Sampler(&sampler),
            ],
        )
    }
}

impl Rendered for PolygonInstance {
    #[inline(always)]
    fn get_id(&self) -> Option<usize> { self.id }
    #[inline(always)]
    fn set_id(&mut self, id: usize) { self.id = Some(id) }

    #[inline(always)]
    fn vertex_buffer(&self, _: &DeviceHandler) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        (
            Arc::clone(&self.polygon.0),
            Some(Arc::clone(&self.polygon.1)),
        )
    }
    #[inline(always)]
    fn bind_group_layout(&self, device_handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        let layout = if self.desc.texture.is_some() {
            self.textured_bdl(&device_handler.device())
        } else {
            self.non_textured_bdl(&device_handler.device())
        };
        Arc::new(layout)
    }
    #[inline(always)]
    fn bind_group(&self, device_handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
        let bind_group = if self.desc.texture.is_some() {
            self.textured_bg(&device_handler.device(), &device_handler.queue(), layout)
        } else {
            self.non_textured_bg(&device_handler.device(), layout)
        };
        Arc::new(bind_group)
    }
    #[inline(always)]
    fn pipeline(
        &self,
        device_handler: &DeviceHandler,
        layout: &PipelineLayout,
    ) -> Arc<RenderPipeline>
    {
        let vertex_shader = include_spirv!("shaders/polygon.vert.spv");
        let fragment_shader = if self.desc.texture.is_some() {
            include_spirv!("shaders/textured_polygon.frag.spv")
        } else {
            include_spirv!("shaders/polygon.frag.spv")
        };
        self.pipeline_with_shader(vertex_shader, fragment_shader, device_handler, layout)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct AttrVertex {
    pub position: [f32; 3],
    pub uv_coord: [f32; 2],
    pub normal: [f32; 3],
}
unsafe impl Zeroable for AttrVertex {}
unsafe impl Pod for AttrVertex {}

#[repr(C)]
#[derive(Debug, Clone)]
struct ExpandedPolygon {
    vertices: Vec<AttrVertex>,
    indices: Vec<u32>,
}

impl ExpandedPolygon {
    pub fn buffers(&self, device: &Device) -> (BufferHandler, BufferHandler) {
        let vertex_buffer = BufferHandler::new(
            device.create_buffer_init(&BufferInitDescriptor {
                contents: bytemuck::cast_slice(&self.vertices),
                usage: BufferUsage::VERTEX,
                label: None,
            }),
            (self.vertices.len() * std::mem::size_of::<AttrVertex>()) as u64,
        );
        let index_buffer = BufferHandler::new(
            device.create_buffer_init(&BufferInitDescriptor {
                contents: bytemuck::cast_slice(&self.indices),
                usage: BufferUsage::INDEX,
                label: None,
            }),
            self.indices.len() as u64,
        );
        (vertex_buffer, index_buffer)
    }
}

fn signup_vertex(
    polymesh: &PolygonMesh,
    vertex: &[usize; 3],
    glpolymesh: &mut ExpandedPolygon,
    vertex_map: &mut HashMap<[usize; 3], u32>,
)
{
    let key = [vertex[0], vertex[1], vertex[2]];
    let idx = match vertex_map.get(&key) {
        Some(idx) => *idx,
        None => {
            let idx = glpolymesh.vertices.len() as u32;
            let position = (&polymesh.positions[key[0]]).cast().unwrap().into();
            let uv_coord = match polymesh.uv_coords.is_empty() {
                true => [0.0, 0.0],
                false => (&polymesh.uv_coords[key[1]]).cast().unwrap().into(),
            };
            let normal = match polymesh.normals.is_empty() {
                true => [0.0, 0.0, 0.0],
                false => (&polymesh.normals[key[2]]).cast().unwrap().into(),
            };
            let wgpuvertex = AttrVertex {
                position,
                uv_coord,
                normal,
            };
            vertex_map.insert(key, idx);
            glpolymesh.vertices.push(wgpuvertex);
            idx
        }
    };
    glpolymesh.indices.push(idx);
}

impl Default for ExpandedPolygon {
    fn default() -> ExpandedPolygon {
        ExpandedPolygon {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl From<&PolygonMesh> for ExpandedPolygon {
    fn from(polymesh: &PolygonMesh) -> ExpandedPolygon {
        let mut glpolymesh = ExpandedPolygon::default();
        let mut vertex_map = HashMap::<[usize; 3], u32>::new();
        for tri in &polymesh.tri_faces {
            signup_vertex(polymesh, &tri[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &tri[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &tri[2], &mut glpolymesh, &mut vertex_map);
        }
        for quad in &polymesh.quad_faces {
            signup_vertex(polymesh, &quad[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[3], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[2], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[3], &mut glpolymesh, &mut vertex_map);
        }
        for face in &polymesh.other_faces {
            for i in 2..face.len() {
                signup_vertex(polymesh, &face[0], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, &face[i - 1], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, &face[i], &mut glpolymesh, &mut vertex_map);
            }
        }
        glpolymesh
    }
}

impl From<&StructuredMesh> for ExpandedPolygon {
    fn from(mesh: &StructuredMesh) -> ExpandedPolygon {
        let mut glpolymesh = ExpandedPolygon::default();
        let (m, n) = (mesh.uv_division.0.len(), mesh.uv_division.1.len());
        for i in 0..m {
            for j in 0..n {
                glpolymesh.vertices.push(AttrVertex {
                    position: mesh.positions[i][j].cast().unwrap().into(),
                    uv_coord: [mesh.uv_division.0[i] as f32, mesh.uv_division.1[j] as f32],
                    normal: mesh.normals[i][j].cast().unwrap().into(),
                });
            }
        }
        for i in 1..m {
            for j in 1..n {
                glpolymesh.indices.push(((i - 1) * n + j - 1) as u32);
                glpolymesh.indices.push((i * n + j - 1) as u32);
                glpolymesh.indices.push(((i - 1) * n + j) as u32);
                glpolymesh.indices.push(((i - 1) * n + j) as u32);
                glpolymesh.indices.push((i * n + j - 1) as u32);
                glpolymesh.indices.push((i * n + j) as u32);
            }
        }
        glpolymesh
    }
}

impl std::fmt::Debug for PolygonInstance {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        f.pad("PolygonInstance {\n")?;
        f.write_fmt(format_args!("  polygon: {:?}\n", self.polygon))?;
        f.write_fmt(format_args!("  matrix: {:?}\n", self.desc.matrix))?;
        f.write_fmt(format_args!("  material: {:?}\n", self.desc.material))?;
        match self.desc.texture {
            Some(_) => f.write_fmt(format_args!("Some(<omitted>)\n}}")),
            None => f.write_fmt(format_args!("None\n}}")),
        }
    }
}
