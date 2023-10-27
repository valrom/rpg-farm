pub mod context;
pub mod buffers;
mod camera;
mod texture;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoopWindowTarget},
    window::Window,
};

use context::Context;


pub struct App {
    context : Context,
    pub camera_angles: cgmath::Point2<f32>,
}


impl App {
    pub async fn new(window: Window) -> Self {
        Self {
            context: Context::new(window).await,
            camera_angles: cgmath::Point2::new(0.0, 0.0),
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
                self.context.camera.set_angles(self.camera_angles);
                self.context.render().unwrap();
            }
            
            Event::MainEventsCleared => {
                self.context.window().request_redraw();
            }

            Event::DeviceEvent {
                event,
                ..
            } => {
                self.handle_device_event(event, control_flow);
            },
            
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

    fn handle_device_event(&mut self, event: DeviceEvent, control_flow: &mut ControlFlow) {
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
                input: KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::Space),
                    state: ElementState::Released,
                    ..
                },
                ..
            } => {
                self.context.is_render_first = !self.context.is_render_first;
                true
            },

            _ => false
        }
    }
}