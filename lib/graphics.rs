use crate::engine;
use crate::platform;
use gl;
use gl::types::*;
use glutin::{
    dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder, Api, ContextBuilder, GlProfile,
    GlRequest, PossiblyCurrent, WindowedContext,
};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Camera {
    pub width: i64,
    pub height: i64,
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

pub struct Context {
    pub windowed_context: WindowedContext<PossiblyCurrent>,
    renderer_started: Instant,
}

impl Context {
    pub fn new(config: &engine::Config, platform: &platform::Platform) -> Self {
        let window_size = LogicalSize::new(config.window_width as f64, config.window_height as f64);

        let window_builder = WindowBuilder::new()
            .with_title(&config.application_name)
            .with_resizable(true) // need to add this to the engine configuration
            .with_inner_size(window_size);

        let windowed_context = unsafe {
            ContextBuilder::new()
                .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
                .with_gl_profile(GlProfile::Core)
                .with_vsync(true)
                .build_windowed(window_builder, &platform.events)
                .expect("graphics context initialisation failed")
                .make_current()
                .expect("failed to access graphics context")
        };

        gl::load_with(|s| windowed_context.get_proc_address(s) as *const _);

        let renderer_started = Instant::now();

        Context {
            windowed_context,
            renderer_started,
        }
    }

    pub fn resize_framebuffer(&mut self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }

    // This is probably temporary, buffers will be swapped after draw calls.
    pub fn swap_buffers(&mut self) {
        self.windowed_context.swap_buffers().unwrap();
        let seconds = self.renderer_started.elapsed().as_secs_f32();

        unsafe {
            gl::ClearColor(seconds.cos(), seconds.sin(), seconds.cos(), 1.);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}
