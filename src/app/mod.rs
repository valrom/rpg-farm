pub mod context;
pub mod buffers;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoopWindowTarget},
    window::Window,
};

use context::Context;


pub struct App {
    context : Context,
}


impl App {
    pub async fn new(window: Window) -> Self {
        Self {
            context: Context::new(window).await
        }
    }

    pub fn main_loop(&mut self, event: Event<()>, _event_loop: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow) {
        self.handle_event(event, control_flow);
    }

    fn handle_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) -> bool {
        match event {
            Event::WindowEvent {
                window_id,
                event
            } if window_id == self.context.window().id() => self.handle_window_event(event, control_flow),
            
            Event::RedrawRequested(window_id) if window_id == self.context.window().id() => {
                self.context.render().unwrap();
            }
            
            Event::MainEventsCleared => {
                self.context.window().request_redraw();
            }
            
            _ => {},
        }

        true
    }

    fn handle_window_event(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        
        if self.input(&event) {
            return;
        }
        
        
        match event {
            WindowEvent::Resized(size) => {
                self.context.resize(size);
            },

            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            
            _ => {},
        }
    }
    
    fn input(&mut self, event: &WindowEvent) -> bool {
        
        if let WindowEvent::KeyboardInput {
            input: KeyboardInput {
                virtual_keycode: Some(VirtualKeyCode::Space),
                state: ElementState::Released,
                ..
            },
            ..
        } = *event {
            self.context.is_render_first = !self.context.is_render_first;
            true
        } else {
            false
        }
    }
}