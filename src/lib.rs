mod app;

use app::App;


use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};
use crate::app::context::Context;
use crate::app::GameLogic;

struct TestLogic;

impl GameLogic for TestLogic {
    fn render<'a, 'b>(&'a self, render_pass: &'b mut wgpu::RenderPass<'a>, context: &'a Context) where 'a : 'b {

        render_pass.set_pipeline(&context.pipeline);
        render_pass.set_bind_group(1, &context.matrix_uniform.bind_group, &[]);

        let texture = if context.is_render_first {
            &context.first_texture
        } else {
            &context.second_texture
        };

        context.mesh.draw(texture, render_pass);
    }
}


pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window =
        WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let mut app = App::new(window, &TestLogic).await;

    event_loop.run(move |event, elwt|{
        app.main_loop(event, elwt);
    }).unwrap();
}