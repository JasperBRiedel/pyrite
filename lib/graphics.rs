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

pub struct Context {
    pub windowed_context: WindowedContext<PossiblyCurrent>,
    renderer_started: Instant,
    framebuffer_size: (f32, f32),
    tileset: Option<Tileset>,
    pub viewport: Viewport,
    scene: Scene,
    quad: Quad,
    shader: Shader,
    pending_render: bool,
}

impl Context {
    pub fn new(config: &engine::Config, platform: &platform::Platform) -> Self {
        let window_size = LogicalSize::new(config.window_width as f64, config.window_height as f64);

        let window_builder = WindowBuilder::new()
            .with_title(&config.application_name)
            .with_resizable(config.window_resizable)
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

        let viewport = Viewport::new(config.viewport_width, config.viewport_height);

        let scene = Scene::new();

        let quad = Quad::new();

        let shader = Shader::new(
            include_str!("pass_through.vert"),
            include_str!("tile_render.frag"),
        );

        let pending_render = true;

        Context {
            windowed_context,
            renderer_started,
            framebuffer_size,
            tileset,
            viewport,
            scene,
            quad,
            shader,
            pending_render,
        }
    }

    pub fn load_tileset(
        &mut self,
        tileset_name: String,
        config: &engine::Config,
        resources: &Box<dyn resources::Provider>,
    ) {
        let image_bytes = resources
            .read_to_bytes(&config.tileset_path)
            .expect("failed to load tileset image");
        let tileset_image = image::load_from_memory(&image_bytes).expect("failed to load tileset");

        match &mut self.tileset {
            Some(existing_tileset) => {
                existing_tileset.load(
                    &tileset_image,
                    (config.tileset_width, config.tileset_height),
                    config.tile_names.clone(),
                );
            }
            None => {
                self.tileset = Some(Tileset::new(
                    &tileset_image,
                    (config.tileset_width, config.tileset_height),
                    config.tile_names.clone(),
                ));
            }
        }
    }

    pub fn set_tile(
        &mut self,
        tile_name: &str,
        x: i32,
        y: i32,
        r: u8,
        g: u8,
        b: u8,
        flip_x: bool,
        flip_y: bool,
    ) {
        if let Some(tileset) = &self.tileset {
            // only allow tiles within the viewport to be changed
            if self.viewport.contains(x, y) {
                let scene_changed = self
                    .scene
                    .set_tile(tileset, tile_name, x, y, r, g, b, flip_x, flip_y);

                // Flag that the scene was changed. Because we only render and swap buffers when
                // there's something new to show.
                self.pending_render = self.pending_render || scene_changed;
            }
        }
    }

    pub fn resize_framebuffer(&mut self, width: i32, height: i32) {
        unsafe {
            self.framebuffer_size = (width as f32, height as f32);
            gl::Viewport(0, 0, width, height);

            // Render the scene at the new resolution
            self.pending_render = true;
        }
    }

    /// Render the scene and present that frame to the screen.
    ///
    /// Only renders if it has been flagged that the scene content changed.
    ///
    /// Returns true if a new frame was rendered and presented.
    pub fn present_frame(&mut self) -> bool {
        // Only render and swap buffers if there's actually something new to show
        if !self.pending_render {
            return false;
        }
        self.pending_render = false;

        let seconds_elapsed = self.renderer_started.elapsed().as_secs_f32();

        if let Some(tileset) = &self.tileset {
            self.clear_frame();

            self.scene.upload();

            unsafe { gl::ActiveTexture(gl::TEXTURE0) };
            tileset.texture.bind();
            unsafe { gl::ActiveTexture(gl::TEXTURE1) };
            self.scene.tiles_texture.bind();
            unsafe { gl::ActiveTexture(gl::TEXTURE2) };
            self.scene.tiles_modifiers_texture.bind();

            self.shader.bind();

            self.shader.set_uniform_1f("time", seconds_elapsed);

            self.shader
                .set_uniform_2f("framebuffer_size", self.framebuffer_size);

            self.shader
                .set_uniform_2f("viewport_size", self.viewport.get_f32());

            // set tileset texture to texture unit 0
            self.shader.set_uniform_1i("tileset", 0);
            self.shader.set_uniform_1i("scene_tiles", 1);
            self.shader.set_uniform_1i("scene_tiles_modifiers", 2);

            self.quad.draw();
        }

        self.windowed_context.swap_buffers().unwrap();

        // We rendered a frame, so return true as per the doc comment.
        return true;
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

#[derive(Clone, Debug)]
pub struct Viewport {
    width: i32,
    height: i32,
}

impl Viewport {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width: width.min(1024),
            height: height.min(1024),
        }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= 0 && x <= self.width && y >= 0 && y <= self.height
    }

    pub fn set(&mut self, width: i32, height: i32) {
        self.width = width.min(1024);
        self.height = height.min(1024);
    }

    pub fn get(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    pub fn get_f32(&self) -> (f32, f32) {
        (self.width as f32, self.height as f32)
    }
}

struct Scene {
    tiles: Vec<(f32, f32)>,
    tiles_upload_buffer: Vec<(f32, f32)>,
    tiles_modifiers: Vec<(u8, u8, u8, u8)>,
    tiles_modifiers_upload_buffer: Vec<(u8, u8, u8, u8)>,

    tiles_texture: Texture,
    tiles_modifiers_texture: Texture,

    upload_pending: bool,
    upload_region_top_left: (u32, u32),
    upload_region_bottom_right: (u32, u32),
}

impl Scene {
    const SCENE_MAX_SIZE: (i32, i32) = (1024, 1024);
    const SCENE_TILE_COUNT: usize = (Self::SCENE_MAX_SIZE.0 * Self::SCENE_MAX_SIZE.1) as usize;

    fn new() -> Self {
        let tiles = vec![(0.0, 0.0); Self::SCENE_TILE_COUNT];
        let tiles_upload_buffer = vec![(0.0, 0.0); Self::SCENE_TILE_COUNT];
        let tiles_modifiers = vec![(255, 255, 255, 0); Self::SCENE_TILE_COUNT];
        let tiles_modifiers_upload_buffer = vec![(255, 255, 255, 0); Self::SCENE_TILE_COUNT];

        // create scene textures and upload scene data
        let tiles_texture =
            Texture::from_vec2_f32(Self::SCENE_MAX_SIZE.0, Self::SCENE_MAX_SIZE.1, &tiles);

        let tiles_modifiers_texture = Texture::from_vec4_u8(
            Self::SCENE_MAX_SIZE.0,
            Self::SCENE_MAX_SIZE.1,
            &tiles_modifiers,
        );

        let upload_pending = false;
        let upload_region_top_left = (1024, 1024);
        let upload_region_bottom_right = (0, 0);

        Self {
            tiles,
            tiles_upload_buffer,
            tiles_modifiers,
            tiles_modifiers_upload_buffer,
            tiles_texture,
            tiles_modifiers_texture,
            upload_pending,
            upload_region_top_left,
            upload_region_bottom_right,
        }
    }

    fn upload(&mut self) {
        if self.upload_pending {
            // calculate update region x and y offset, and width and height
            let update_region_xy_wh = self.get_update_region();

            self.copy_update_region_to_upload_buffers(update_region_xy_wh);

            // preform partial update
            self.tiles_texture.partial_update_from_vec2_f32(
                update_region_xy_wh.0,
                update_region_xy_wh.1,
                update_region_xy_wh.2,
                update_region_xy_wh.3,
                &self.tiles_upload_buffer,
            );

            self.tiles_modifiers_texture.partial_update_from_vec4_u8(
                update_region_xy_wh.0,
                update_region_xy_wh.1,
                update_region_xy_wh.2,
                update_region_xy_wh.3,
                &self.tiles_modifiers_upload_buffer,
            );

            // reset update region tracking
            self.upload_pending = false;
            self.upload_region_top_left = (1024, 1024);
            self.upload_region_bottom_right = (0, 0);
        }
    }

    fn get_update_region(&self) -> (i32, i32, i32, i32) {
        (
            self.upload_region_top_left.0 as i32,
            self.upload_region_top_left.1 as i32,
            (self.upload_region_bottom_right.0 - self.upload_region_top_left.0 + 1) as i32,
            (self.upload_region_bottom_right.1 - self.upload_region_top_left.1 + 1) as i32,
        )
    }

    fn copy_update_region_to_upload_buffers(&mut self, region: (i32, i32, i32, i32)) {
        let region = (
            region.0 as u32,
            region.1 as u32,
            region.2 as u32,
            region.3 as u32,
        );

        for local_x in 0..region.2 {
            for local_y in 0..region.3 {
                let global_x = local_x + region.0;
                let global_y = local_y + region.1;

                let local_index = (local_y * region.2 + local_x);
                let global_index = (global_y * Self::SCENE_MAX_SIZE.0 as u32 + global_x);

                self.tiles_upload_buffer[local_index as usize] = self.tiles[global_index as usize];
                self.tiles_modifiers_upload_buffer[local_index as usize] =
                    self.tiles_modifiers[global_index as usize];
            }
        }
    }

    /// Returns true if the scene was actually modified
    fn set_tile(
        &mut self,
        tileset: &Tileset,
        name: &str,
        x: i32,
        y: i32,
        r: u8,
        g: u8,
        b: u8,
        flip_x: bool,
        flip_y: bool,
    ) -> bool {
        // we don't care about negative locations, but it makes easier for other systems to
        // interact when we accept a signed number, so we convert here.
        let x = x as u32;
        let y = y as u32;

        // find liner index
        let index = (y * Self::SCENE_MAX_SIZE.0 as u32 + x) as usize;

        // determine flip value
        let flip = match (flip_x, flip_y) {
            (false, false) => 0,  // flip none = 0
            (true, false) => 51,  // flip x = 0.2
            (false, true) => 102, // flip y = 0.4
            (true, true) => 153,  // flip x and y = .6
        };

        // if all the required resources are available, we preform a tile update
        match (
            tileset.get_tile_texture_location(name),
            self.tiles.get_mut(index),
            self.tiles_modifiers.get_mut(index),
        ) {
            (Some(tile_texture_location), Some(tile), Some(modifiers)) => {
                let pending_modifiers = (r, g, b, flip);

                // we should update the data only if the new data is different
                let should_update_data =
                    *tile != tile_texture_location || *modifiers != pending_modifiers;

                if should_update_data {
                    *tile = tile_texture_location;
                    *modifiers = pending_modifiers;

                    self.upload_region_top_left = (
                        self.upload_region_top_left.0.min(x),
                        self.upload_region_top_left.1.min(y),
                    );

                    self.upload_region_bottom_right = (
                        self.upload_region_bottom_right.0.max(x),
                        self.upload_region_bottom_right.1.max(y),
                    );

                    self.upload_pending = true;

                    return true;
                }
            }
            _ => (),
        };

        return false;
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

    fn get_tile_texture_location(&self, tile_name: &str) -> Option<(f32, f32)> {
        Some((0.0, 0.0))
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

            let pixels: Vec<u8> = image.to_rgba().into_raw();

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

            let pixels: Vec<u8> = image.to_rgba().into_raw();

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

    fn from_vec2_f32(width: i32, height: i32, data: &[(f32, f32)]) -> Self {
        unsafe {
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);

            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // could be a problem with the internal format here
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RG as i32,
                width,
                height,
                0,
                gl::RG,
                gl::FLOAT,
                std::mem::transmute(&data[0]),
            );

            if texture <= 0 {
                panic!("texture creation failed");
            }

            Self { texture }
        }
    }

    fn update_from_vec2_f32(&mut self, width: i32, height: i32, data: &[(f32, f32)]) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // could be a problem with the internal format here
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RG as i32,
                width,
                height,
                0,
                gl::RG,
                gl::FLOAT,
                std::mem::transmute(&data[0]),
            );
        }
    }

    fn partial_update_from_vec2_f32(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        data: &[(f32, f32)],
    ) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x,
                y,
                width,
                height,
                gl::RG,
                gl::FLOAT,
                std::mem::transmute(&data[0]),
            );
        }
    }

    fn from_vec4_u8(width: i32, height: i32, data: &[(u8, u8, u8, u8)]) -> Self {
        unsafe {
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);

            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::mem::transmute(&data[0]),
            );

            if texture <= 0 {
                panic!("texture creation failed");
            }

            Self { texture }
        }
    }

    fn update_from_vec4_u8(&mut self, width: i32, height: i32, data: &[(u8, u8, u8, u8)]) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::mem::transmute(&data[0]),
            );
        }
    }

    fn partial_update_from_vec4_u8(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        data: &[(u8, u8, u8, u8)],
    ) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x,
                y,
                width,
                height,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::mem::transmute(&data[0]),
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
        1., 1., 1., 0., // top right
        1., -1., 1., 1., // bottom right
        -1., -1., 0., 1., // bottom left
        -1., 1., 0., 0., // top left
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
