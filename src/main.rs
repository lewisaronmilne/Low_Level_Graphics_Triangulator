use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use wgpu::util::DeviceExt;

use std::sync::Arc;

mod triangulator;
mod data;

#[derive(Debug, Copy, Clone)]
pub struct XY 
{ 
    pub x: f32, 
    pub y: f32 
}

impl XY 
{
    pub fn new(x: f32, y: f32) -> XY { XY {x, y} }
    pub fn init() -> XY { XY::new(0.0, 0.0) }
}

const CLEAR_COLOUR: wgpu::Color = wgpu::Color { r: 0.1, g: 0.1, b: 0.2, a: 1.0 };

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex 
{
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex 
{
    fn desc() -> wgpu::VertexBufferLayout<'static> 
    {
        wgpu::VertexBufferLayout
        {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: 
            &[
                wgpu::VertexAttribute
                {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute
                {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }
}

struct State 
{
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    window: Arc<Window>,
    shapes_list: Vec<Vec<Vertex>>,
    shape_num: u32,
}

impl State 
{
    async fn new(window: Arc<Window>) -> Self
    {
        let size = window.inner_size();

        let instance_descriptor = wgpu::InstanceDescriptor 
        {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        };
        let instance = wgpu::Instance::new(&instance_descriptor);

        let surface = instance.create_surface(window.clone()).unwrap();

        let request_adaptor_options = wgpu::RequestAdapterOptions
        {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        let adapter = instance.request_adapter(&request_adaptor_options).await.unwrap();

        let device_descriptor = wgpu::DeviceDescriptor
        {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off,
        };
        let (device, queue) = adapter.request_device(&device_descriptor).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration 
        {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let shader_module_descriptor = wgpu::ShaderModuleDescriptor 
        {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        };
        let shader = device.create_shader_module(shader_module_descriptor);

        let pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor 
        {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        };
        let render_pipeline_layout = device.create_pipeline_layout(&pipeline_layout_descriptor);

        let vertex = wgpu::VertexState 
        {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[ Vertex::desc() ],
            compilation_options: Default::default(),
        };
        let fragment_color_target = wgpu::ColorTargetState 
        {
            format: config.format,
            blend: Some(wgpu::BlendState { color: wgpu::BlendComponent::REPLACE, alpha: wgpu::BlendComponent::REPLACE }),
            write_mask: wgpu::ColorWrites::ALL,
        };
        let fragment = wgpu::FragmentState 
        {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(fragment_color_target)],
            compilation_options: Default::default(),
        };
        let pipeline_descriptor = &wgpu::RenderPipelineDescriptor 
        {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: vertex,
            fragment: Some(fragment),
            primitive: wgpu::PrimitiveState 
            {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState
            {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        };
        let render_pipeline = device.create_render_pipeline(pipeline_descriptor);
        
        let shapes_list = data::make_shapes();
        let shape_num = 0;

        let vertex_buffer_init_descriptor = wgpu::util::BufferInitDescriptor 
        {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&shapes_list[0]),
            usage: wgpu::BufferUsages::VERTEX,
        };
        let vertex_buffer = device.create_buffer_init(&vertex_buffer_init_descriptor);
        let num_vertices = shapes_list[0].len() as u32;

        Self
        {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            render_pipeline,
            vertex_buffer,
            num_vertices,
            window,
            shapes_list, 
            shape_num
        }
    }

    fn resize(&mut self, width: u32, height: u32)
    {
        if width == 0 || height == 0 { return; }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.is_surface_configured = true;
    }

    fn change_vertices(&mut self)
    {
        self.shape_num = (self.shape_num + 1) % self.shapes_list.len() as u32;

        let verts = &self.shapes_list[self.shape_num as usize];

        let vertex_buffer_init_descriptor = wgpu::util::BufferInitDescriptor 
        {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        };
        self.vertex_buffer = self.device.create_buffer_init(&vertex_buffer_init_descriptor);
        self.num_vertices = verts.len() as u32;
    }

    fn render(&mut self) -> ()
    {
        self.window.request_redraw();

        if !self.is_surface_configured { return; }

        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

        let color_attachment = wgpu::RenderPassColorAttachment 
        {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations 
            {
                load: wgpu::LoadOp::Clear(CLEAR_COLOUR),
                store: wgpu::StoreOp::Store,
            },
        };
        let render_pass_descriptor = wgpu::RenderPassDescriptor 
        {
            label: Some("Render Pass"),
            color_attachments: &[ Some(color_attachment) ],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        };
        let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.num_vertices, 0..1);

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, key_code: KeyCode, is_pressed: bool) 
    {
        match (key_code, is_pressed)
        {
            (KeyCode::Escape, true) => event_loop.exit(),
            (KeyCode::Space, true) => self.change_vertices(),
            _ => {}
        }
    }
}

struct App 
{
    state: Option<State>
}

impl App 
{
    fn new() -> Self
    {
        App { state: None }
    }
}

impl ApplicationHandler for App
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop)
    {
        let window_attr = Window::default_attributes().with_title("Low Level Graphics Triangulator!");
        let new_window = Arc::new(event_loop.create_window(window_attr).unwrap());
        self.state = Some(pollster::block_on(State::new(new_window)))
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent)
    {
        let state = self.state.as_mut().unwrap();
        
        match event 
        {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => state.render(),
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(key_code), state: key_state, .. }, .. } => 
            {
                state.handle_key(event_loop, key_code, key_state.is_pressed());
            }
            _ => (),
        }
    }
}

fn main() 
{
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();

    let _loop_result = event_loop.run_app(&mut app);
}