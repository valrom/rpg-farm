mod app;

use cgmath::SquareMatrix;
use app::App;


use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};
use crate::app::buffers::{INDICES, VERTICES};
use crate::app::context::{DrawCall, Renderer};
use crate::app::GameLogic;

struct TestLogic {
    texture: Option<usize>,
    mesh: Option<usize>,
}

impl TestLogic {
    fn new() -> Self {
        Self {
            texture: None,
            mesh: None,
        }
    }
}

impl GameLogic for TestLogic {
    fn init<'a, 'b>(&'a mut self, renderer: &'b mut Renderer<'a>) where 'a: 'b {
        self.mesh = renderer.add_mesh(VERTICES, INDICES);
        self.texture = renderer.add_texture("resources/grass.jpeg");
    }

    fn render<'a, 'b>(&'a mut self, renderer: &'b mut Renderer<'a>) where 'a: 'b {

        if let (Some(mesh_id), Some(texture_id)) = (self.mesh, self.texture) {
            renderer.draw(DrawCall {
                mesh_id,
                texture_id,
                matrix: cgmath::Matrix4::<f32>::identity(),
            })
        }
    }
}


pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window =
        WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let mut test_logic = TestLogic::new();

    let mut app = App::new(window, &mut test_logic).await;

    event_loop.run(move |event, elwt|{
        app.main_loop(event, elwt);
    }).unwrap();
}