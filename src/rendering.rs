use crate::{XY, Colour};

pub struct Renderer
{
    device: wgpu::Device, 
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup, 
    bind_group_layout: wgpu::BindGroupLayout,
    vertex_buffer: wgpu::Buffer, 
    vertex_count: u32,
    swap_chain: wgpu::SwapChain,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
    surface: wgpu::Surface,
    clear_colour: wgpu::Color,
}

impl Renderer
{
    pub fn create(window: &winit::window::Window) -> Renderer
    {
        let instance = wgpu::Instance::new();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions 
        {
            power_preference: wgpu::PowerPreference::LowPower
        });
        let device = adapter.request_device(&wgpu::DeviceDescriptor 
        {
            extensions: wgpu::Extensions { anisotropic_filtering: false, },
            limits: wgpu::Limits::default()
        });

        let vert_spirv = include_bytes!("../shaders/vert.spirv");
        let frag_spirv = include_bytes!("../shaders/frag.spirv");
        let vert_shader = device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vert_spirv[..])).unwrap());
        let frag_shader = device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&frag_spirv[..])).unwrap());

        let vertex_buffer = device
            .create_buffer_mapped(0, wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&[] as &[[f32; 5]]);
        let vertex_count = 0u32;

        let (render_width, render_height) = window
            .inner_size()
            .to_physical(window.hidpi_factor())
            .into();
        let clear_colour = wgpu::Color{r:0.0, b:0.0, g:0.0, a:0.8};

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        { 
            bindings: 
            &[
                wgpu::BindGroupLayoutBinding 
                {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                wgpu::BindGroupLayoutBinding 
                {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::StorageBuffer { dynamic: false, readonly: true },
                }
            ]
        });

        let bind_group = Renderer::create_bind_group(&device, &bind_group_layout, render_width, render_height, clear_colour);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        { 
            bind_group_layouts: &[&bind_group_layout] 
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor 
        {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor{ module: &vert_shader, entry_point: "main" },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor{ module: &frag_shader, entry_point: "main" }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor 
            {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor 
            {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor
            {
                stride: 5 * 4, // 4 is size of f32
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: 
                &[
                    wgpu::VertexAttributeDescriptor 
                    {
                        format: wgpu::VertexFormat::Float2,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor 
                    {
                        format: wgpu::VertexFormat::Float3,
                        offset: 2 * 4,
                        shader_location: 1,
                    },
                ],
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        use raw_window_handle::HasRawWindowHandle;
        let surface = instance.create_surface(window.raw_window_handle());
        let swap_chain_descriptor = wgpu::SwapChainDescriptor 
        {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: render_width,
            height: render_height,
            present_mode: wgpu::PresentMode::Vsync,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

        Renderer 
        { 
            device, render_pipeline, bind_group, bind_group_layout, vertex_buffer, vertex_count,
            swap_chain, surface, swap_chain_descriptor, clear_colour
        }
    }

    pub fn draw(&mut self)
    {
        let frame = self.swap_chain.get_next_texture();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor 
            {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor 
                {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: self.clear_colour
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffers(0, &[(&self.vertex_buffer, 0)]);
            rpass.draw(0..self.vertex_count, 0..1)
        }
        &self.device.get_queue().submit(&[encoder.finish()]);
    }

    pub fn set_triangles(&mut self, triangles: &[(XY, XY, XY)])
    {
        let mut vertices = Vec::with_capacity(triangles.len());
        for t in triangles
        { 
            let colour = Colour::random();
            vertices.push([t.0.x, t.0.y, colour.r, colour.g, colour.b]);
            vertices.push([t.1.x, t.1.y, colour.r, colour.g, colour.b]);
            vertices.push([t.2.x, t.2.y, colour.r, colour.g, colour.b]);
        };

        self.vertex_buffer = self.device
            .create_buffer_mapped(vertices.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertices);

        self.vertex_count = vertices.len() as u32;
    }

    fn create_bind_group
    (
        in_device: &wgpu::Device, 
        in_layout: &wgpu::BindGroupLayout,
        render_width: u32, 
        render_height: u32,
        clear_colour: wgpu::Color
    ) -> wgpu::BindGroup
    {
        let uniforms_buffer = in_device
            .create_buffer_mapped(1, wgpu::BufferUsage::STORAGE_READ)
            .fill_from_slice(&[(render_width, render_height, clear_colour.r as f32, clear_colour.g as f32, clear_colour.b as f32, clear_colour.a as f32)]);

        let masks = in_device
            .create_buffer_mapped(4, wgpu::BufferUsage::STORAGE_READ | wgpu::BufferUsage::MAP_WRITE)
            .fill_from_slice(&[-0.5, -0.5, 1.0, 1.0 as f32]);

        return in_device.create_bind_group(&wgpu::BindGroupDescriptor
        { 
            layout: in_layout, 
            bindings: 
            &[
                wgpu::Binding 
                {
                    binding: 0 as u32,
                    resource: wgpu::BindingResource::Buffer { buffer: &uniforms_buffer, range: 0..24 }
                },
                wgpu::Binding 
                {
                    binding: 1 as u32,
                    resource: wgpu::BindingResource::Buffer { buffer: &masks, range: 0..16 }
                },
            ]   
        });
    }

    pub fn update_render_dimensions(&mut self, render_width: u32, render_height: u32)
    {
        self.swap_chain_descriptor.width = render_width;
        self.swap_chain_descriptor.height = render_height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_descriptor);
        self.bind_group = Renderer::create_bind_group(&self.device, &self.bind_group_layout, render_width, render_height, self.clear_colour)
    }
}