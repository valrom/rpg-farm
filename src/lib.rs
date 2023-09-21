mod app;

use app::App;


use winit::{
    event_loop::EventLoop,
    window::WindowBuilder,
};


pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window =
        WindowBuilder::new()
        .build(&event_loop)
        .unwrap();


    let mut app = App::new(window).await;

    event_loop.run(move |event, event_lp, control_flow|{
        app.main_loop(event, event_lp, control_flow);
    });
}