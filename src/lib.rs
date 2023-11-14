mod app;

use cgmath::{SquareMatrix, Vector4};
use app::App;


use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};
use crate::app::buffers::{INDICES, VERTICES};
use crate::app::context::{DrawCall, Renderer};
use crate::app::GameLogic;

struct TestLogic {
    textures: Vec<usize>,
    mesh: usize,
}

impl TestLogic {
    fn new() -> Self {
        Self {
            textures: Vec::with_capacity(2),
            mesh: 0,
        }
    }
}

impl GameLogic for TestLogic {
    fn init<'a, 'b>(&'a mut self, renderer: &'b mut Renderer<'a>) where 'a: 'b {
        self.mesh = renderer.add_mesh(VERTICES, INDICES);

        let first_texture = renderer.add_texture("resources/grass.jpeg")
            .expect("Can't first loading texture");

        let second_texture = renderer.add_texture("resources/stone.jpeg")
            .expect("Can't loading second texture");

        self.textures.push(first_texture);
        self.textures.push(second_texture);
    }

    fn render<'a, 'b>(&'a mut self, renderer: &'b mut Renderer<'a>) where 'a: 'b {
        let mut matrix = cgmath::Matrix4::<f32>::identity();

        const DISTANCE : f32 = 1.25;

        for x in -3..=3 {
            for y in -3..=3 {
                for z in -3..=3 {
                    matrix.w = Vector4::new(
                        DISTANCE * x as f32,
                        DISTANCE * y as f32,
                        DISTANCE * z as f32,
                        1.0,
                    );

                    renderer.draw(DrawCall {
                        mesh_id: self.mesh,
                        texture_id: self.textures[(x + y + z) as usize % self.textures.len()],
                        matrix,
                    });
                }
            }
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