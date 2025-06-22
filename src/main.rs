mod rendering;
mod triangulator;
mod data;

#[derive(Debug, Copy, Clone)]
pub struct XY { pub x: f32, pub y: f32 }
impl XY 
{
    pub fn new(x: f32, y: f32) -> XY { XY {x, y} }
    pub fn init() -> XY { XY::new(0.0, 0.0) }
}

#[derive(Debug, Clone)]
pub struct Colour { pub r: f32, pub g: f32, pub b: f32 }
impl Colour
{ 
    pub fn new(r: f32, g: f32, b: f32) -> Colour { Colour {r, g, b} }
    pub fn random() -> Colour
    {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Colour::new(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>())
    }
}

fn main() 
{
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Shite Graphics!")
        .with_inner_size(winit::dpi::LogicalSize{ width: 1000.0, height: 1000.0 })
        .with_visible(false)
        .with_transparent(true)
        //.with_decorations(false)
        .build(&event_loop)
        .unwrap();
    let mut renderer = rendering::Renderer::create(&window);
    
    let triangulated_paths: Vec<Vec<(XY, XY, XY)>> = data::make_paths()
        .iter()
        .map(|path| triangulator::calc(&path))
        .collect();
    
    let mut current_path_index = 0;
    renderer.set_triangles(&triangulated_paths[current_path_index]);

    window.set_visible(true);

    event_loop.run(move |event, _, control_flow| 
    {
        use winit::{ event, event::WindowEvent, event_loop::ControlFlow};

        *control_flow = ControlFlow::Wait; 

        match event
        {
            event::Event::WindowEvent { event, .. } => match event
            {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(window_size) => 
                {
                    let (width, height) = window_size.to_physical(window.hidpi_factor()).into();
                    renderer.update_render_dimensions(width, height);
                },
                WindowEvent::MouseInput{ button: event::MouseButton::Left, state: event::ElementState::Pressed, .. } =>
                {
                    current_path_index = (current_path_index + 1) % triangulated_paths.len();
                    renderer.set_triangles(&triangulated_paths[current_path_index]);
                },
                _ => (),
            },
            event::Event::EventsCleared => renderer.draw(),
            _ => (),
        }
    });
}