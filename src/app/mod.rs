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
use winit::keyboard::PhysicalKey;

use context::Context;
use crate::app::context::Renderer;


pub struct App<'a> {
    game_logic: &'a mut dyn GameLogic,
    context : Context,
    inputs: Vec<PhysicalKey>,
}


pub trait GameLogic {
    fn render<'a, 'b>(&'a mut self, renderer: &'b mut Renderer<'a>) where 'a : 'b;

    fn init<'a, 'b>(&'a mut self, renderer: &'b mut Renderer<'a>) where 'a : 'b;

    fn input(&mut self, inputs: Vec<PhysicalKey>);
}

impl<'a> App<'a> {
    pub async fn new(window: Window, game_logic: &'a mut dyn GameLogic) -> App<'a> {

        let mut context = Context::new(window).await;
        context.init(game_logic);

        Self {
            game_logic,
            context,
            inputs: Vec::new(),
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
                        self.game_logic.input(std::mem::take(&mut self.inputs));
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

            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state: ElementState::Pressed,
                    physical_key,
                    ..
                },
                ..
            } => {
                self.inputs.push(physical_key);
            }
            
            _ => {},
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent, _elwt: &EventLoopWindowTarget<()>) {
        match event {
            _ => {}
        }
    }
    
    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }
}