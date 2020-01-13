use crate::engine;
use crate::platform;
use crate::resources;
use gl;
use gl::types::*;
use glutin::{
    dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder, Api, ContextBuilder, GlProfile,
    GlRequest, PossiblyCurrent, WindowedContext,
};
use std::ffi;
use std::mem;
use std::ptr;
use std::str;
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
    framebuffer_size: (f32, f32),
    tileset: Option<Tileset>,
    quad: Quad,
    shader: Shader,
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

        let framebuffer_size = (window_size.width as f32, window_size.height as f32);

        let tileset = None;

        let quad = Quad::new();

        let shader = Shader::new(
            include_str!("pass_through.vert"),
            include_str!("tile_render.frag"),
        );

        Context {
            windowed_context,
            renderer_started,
            framebuffer_size,
            tileset,
            quad,
            shader,
        }
    }

    pub fn load_tileset(
        &mut self,
        tileset_name: String,
        config: &engine::Config,
        resources: &Box<dyn resources::Provider>,
    ) {
        let tileset_config = config.tiles.get(&tileset_name).cloned().unwrap();

        let image_bytes = resources
            .read_to_bytes(&format!("tilesets/{}", tileset_config.filename))
            .expect("failed to load tileset image");
        let tileset_image = image::load_from_memory(&image_bytes).expect("failed to load tileset");

        match &mut self.tileset {
            Some(existing_tileset) => {
                existing_tileset.load(
                    &tileset_image,
                    (
                        tileset_config.horizontal_tiles,
                        tileset_config.vertical_tiles,
                    ),
                    tileset_config.tile_names,
                );
            }
            None => {
                self.tileset = Some(Tileset::new(
                    &tileset_image,
                    (
                        tileset_config.horizontal_tiles,
                        tileset_config.vertical_tiles,
                    ),
                    tileset_config.tile_names,
                ));
            }
        }
    }

    pub fn resize_framebuffer(&mut self, width: i32, height: i32) {
        unsafe {
            self.framebuffer_size = (width as f32, height as f32);
            gl::Viewport(0, 0, width, height);
        }
    }

    pub fn present_frame(&self) {
        let seconds_elapsed = self.renderer_started.elapsed().as_secs_f32();

        if let Some(tileset) = &self.tileset {
            self.clear_frame();

            unsafe { gl::ActiveTexture(gl::TEXTURE0) };
            tileset.texture.bind();

            self.shader.bind();
            self.shader.set_uniform_1f("time", seconds_elapsed);
            self.shader
                .set_uniform_2f("framebuffer_size", self.framebuffer_size);

            // set tileset texture to texture unit 0
            self.shader.set_uniform_1i("tileset", 0);

            self.quad.draw();
        }

        self.windowed_context.swap_buffers().unwrap();
    }

    fn clear_frame(&self) {
        unsafe {
            gl::ClearColor(1., 1., 1., 1.);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn clean_up(self) {
        self.quad.clean_up();
    }
}

struct Tileset {
    pub texture: Texture,
}

impl Tileset {
    fn new(
        image: &image::DynamicImage,
        tileset_dimensions: (u32, u32),
        tile_names: Vec<String>,
    ) -> Self {
        let texture = Texture::from_image(image);

        Self { texture }
    }

    fn load(
        &mut self,
        image: &image::DynamicImage,
        tileset_dimensions: (u32, u32),
        tile_names: Vec<String>,
    ) {
        unimplemented!()
    }
}

pub struct Texture {
    texture: u32,
}

impl Texture {
    fn from_image(image: &image::DynamicImage) -> Self {
        unsafe {
            use image::{GenericImage, GenericImageView, Pixel};

            let mut texture = 0;
            gl::GenTextures(1, &mut texture);

            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            let pixels: Vec<u8> = image.flipv().to_rgba().into_raw();

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                image.width() as i32,
                image.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::mem::transmute(&pixels.as_slice()[0]),
            );

            if texture <= 0 {
                panic!("texture creation failed");
            }

            Self { texture }
        }
    }

    fn update_from_image(&mut self, image: &image::DynamicImage) {
        use image::{GenericImage, GenericImageView, Pixel};

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            let pixels: Vec<u8> = image.flipv().to_rgba().into_raw();

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                image.width() as i32,
                image.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::mem::transmute(&pixels.as_slice()[0]),
            );
        }
    }

    fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
        }
    }
}

pub struct Shader {
    program: u32,
}

impl Shader {
    pub fn new(vertex_shader_source: &str, fragment_shader_source: &str) -> Self {
        unsafe {
            let vertex_shader = Self::compile_shader(vertex_shader_source, gl::VERTEX_SHADER);
            let fragment_shader = Self::compile_shader(fragment_shader_source, gl::FRAGMENT_SHADER);

            let program = Self::link_shaders(vertex_shader, fragment_shader);

            Self { program }
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }

    pub fn set_uniform_1i(&self, name: &str, value: i32) {
        unsafe {
            let name = ffi::CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.program, name.as_ptr());

            gl::Uniform1i(location, value);
        }
    }

    pub fn set_uniform_1f(&self, name: &str, value: f32) {
        unsafe {
            let name = ffi::CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.program, name.as_ptr());

            gl::Uniform1f(location, value);
        }
    }

    pub fn set_uniform_2f(&self, name: &str, value: (f32, f32)) {
        unsafe {
            let name = ffi::CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.program, name.as_ptr());

            gl::Uniform2f(location, value.0, value.1);
        }
    }

    pub fn set_uniform_3f(&self, name: &str, value: (f32, f32, f32)) {
        unsafe {
            let name = ffi::CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.program, name.as_ptr());

            gl::Uniform3f(location, value.0, value.1, value.2);
        }
    }

    unsafe fn compile_shader(source: &str, shader_type: GLuint) -> u32 {
        let shader = gl::CreateShader(shader_type);

        let c_str = ffi::CString::new(source.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut compile_status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status);
        if compile_status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            panic!(
                "failed to compile shader: {}",
                str::from_utf8(&buf).expect("failed to decode error message")
            );
        }

        return shader;
    }

    unsafe fn link_shaders(vertex_shader: u32, fragment_shader: u32) -> u32 {
        let program = gl::CreateProgram();

        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);

        gl::LinkProgram(program);

        let mut link_status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut link_status);
        if link_status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            panic!(
                "failed to link shaders: {}",
                str::from_utf8(&buf).expect("failed to read error message")
            );
        };

        return program;
    }
}

pub struct Quad {
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl Quad {
    const QUAD_VERTS: [f32; 16] = [
        1., 1., 1., 1., // top right
        1., -1., 1., 0., // bottom right
        -1., -1., 0., 0., // bottom left
        -1., 1., 0., 1., // top left
    ];

    const QUAD_INDICES: [i32; 6] = [
        0, 1, 3, // first triangle
        1, 2, 3, // second triangle
    ];

    fn new() -> Self {
        let (mut vao, mut vbo, mut ebo) = (0, 0, 0);

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (Self::QUAD_VERTS.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                mem::transmute(&Self::QUAD_VERTS[0]),
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (Self::QUAD_INDICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                mem::transmute(&Self::QUAD_INDICES[0]),
                gl::STATIC_DRAW,
            );

            let stride = 4 * mem::size_of::<GLfloat>() as GLsizei;

            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(0);

            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                mem::transmute(2 * mem::size_of::<GLfloat>()),
            );
            gl::EnableVertexAttribArray(1);
        }

        Self { vao, vbo, ebo }
    }

    fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }
    }

    fn clean_up(self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}
