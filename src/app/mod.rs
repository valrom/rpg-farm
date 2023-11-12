pub mod context;
pub mod buffers;
mod camera;
mod texture;
mod matrix;

use winit::{
    event::*,
    event_loop::EventLoopWindowTarget,
    window::Window,
};
use winit::keyboard::{KeyCode, PhysicalKey};

use context::Context;
use crate::app::context::Renderer;


pub struct App<'a> {
    game_logic: &'a mut dyn GameLogic,
    context : Context,
    pub camera_angles: cgmath::Point2<f32>,
}


pub trait GameLogic {
    fn render<'a, 'b>(&'a mut self, renderer: &'b mut Renderer<'a>) where 'a : 'b;

    fn init<'a, 'b>(&'a mut self, renderer: &'b mut Renderer<'a>) where 'a : 'b;
}

impl<'a> App<'a> {
    pub async fn new(window: Window, game_logic: &'a mut dyn GameLogic) -> App<'a> {

        let mut context = Context::new(window).await;
        context.init(game_logic);

        Self {
            game_logic,
            context,
            camera_angles: cgmath::Point2::new(0.0, 0.0),
        }
    }

    pub fn main_loop(&mut self, event: Event<()>, elwt: &EventLoopWindowTarget<()>) {
        self.handle_event(event, elwt);
    }

    fn handle_event(&mut self, event: Event<()>, elwt: &EventLoopWindowTarget<()>) -> bool {
        match event {
            Event::WindowEvent {
                window_id,
                event
            } if window_id == self.context.window().id() => {

                match event {
                    WindowEvent::RedrawRequested => {
                        self.context.camera.set_angles(self.camera_angles);
                        self.context.render(self.game_logic).unwrap();
                    },

                    _ => self.handle_window_event(event, elwt),
                }

            },
            
            Event::AboutToWait => {
                self.context.window().request_redraw();
            }

            Event::DeviceEvent {
                event,
                ..
            } => {
                self.handle_device_event(event, elwt);
            },
            
            _ => {},
        }

        true
    }

    fn handle_window_event(&mut self, event: WindowEvent, elwt: &EventLoopWindowTarget<()>) {
        
        if self.input(&event) {
            return;
        }
        
        
        match event {
            WindowEvent::Resized(size) => {
                self.context.resize(size);
            },

            WindowEvent::CloseRequested => elwt.exit(),
            
            _ => {},
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent, _elwt: &EventLoopWindowTarget<()>) {
        match event {
            DeviceEvent::MouseMotion {
                delta
            } => {
                self.camera_angles += (delta.0 as f32 / 10.0, delta.1 as f32  / 10.0).into();
            },

            _ => {}
        }
    }
    
    fn input(&mut self, event: &WindowEvent) -> bool {

        match event {
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Space),
                    state: ElementState::Released,
                    ..
                },
                ..
            } => {
                println!("Space pressed");
                true
            },

            _ => false
        }
    }
}