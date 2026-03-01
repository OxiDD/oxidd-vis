use web_sys::{WebGl2RenderingContext as Gl, WebGlFramebuffer, WebGlTexture, WebGlUniformLocation};

pub trait RenderTarget {
    fn bind_buffer(&self, context: &Gl);
    fn clear(&self, context: &Gl);
    fn get_size(&self) -> (usize, usize);
}

pub struct RenderTexture {
    framebuffer: WebGlFramebuffer,
    texture: WebGlTexture,
    color: (f32, f32, f32, f32),
    width: usize,
    height: usize,
}

impl RenderTexture {
    pub fn new(
        context: &Gl,
        width: usize,
        height: usize,
        color: (f32, f32, f32, f32),
    ) -> Option<RenderTexture> {
        let texture = context.create_texture();
        context.bind_texture(Gl::TEXTURE_2D, texture.as_ref());
        context
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                Gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                Gl::RGBA,
                Gl::UNSIGNED_BYTE,
                None,
            )
            .ok()?;
        context.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::LINEAR as i32);
        context.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::LINEAR as i32);
        context.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
        context.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);

        let framebuffer = context.create_framebuffer();
        context.bind_framebuffer(Gl::FRAMEBUFFER, Some(&framebuffer.clone()?));

        context.framebuffer_texture_2d(
            Gl::FRAMEBUFFER,
            Gl::COLOR_ATTACHMENT0,
            Gl::TEXTURE_2D,
            texture.as_ref(),
            0,
        );

        Some(RenderTexture {
            framebuffer: framebuffer?,
            texture: texture?,
            width,
            height,
            color,
        })
    }

    pub fn dispose(&self, context: &Gl) {
        context.delete_framebuffer(Some(&self.framebuffer));
        context.delete_texture(Some(&self.texture));
    }

    pub fn bind_texture(
        &self,
        context: &Gl,
        uniform_location: Option<&WebGlUniformLocation>,
        slot: u8,
    ) {
        context.active_texture(Gl::TEXTURE0 + (slot as u32));
        context.bind_texture(Gl::TEXTURE_2D, Some(&self.texture));
        context.uniform1i(uniform_location, slot as i32);
    }

    pub fn get_pixels(&self, context: &Gl) -> Vec<u8> {
        self.bind_buffer(context);
        let length = 4 * self.width * self.height;
        let mut out = vec![0 as u8; length];
        context
            .read_pixels_with_u8_array_and_dst_offset(
                0,
                0,
                self.width as i32,
                self.height as i32,
                Gl::RGBA,
                Gl::UNSIGNED_BYTE,
                &mut out[0..length],
                0,
            )
            .unwrap();
        out
    }
}
impl RenderTarget for RenderTexture {
    fn bind_buffer(&self, context: &Gl) {
        context.viewport(0, 0, self.width as i32, self.height as i32);
        context.bind_framebuffer(Gl::FRAMEBUFFER, Some(&self.framebuffer));
    }
    fn clear(&self, context: &Gl) {
        self.bind_buffer(context);
        context.clear_color(self.color.0, self.color.1, self.color.2, self.color.3);
        context.clear(Gl::COLOR_BUFFER_BIT);
    }

    fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

// Screen texture
pub struct ScreenTexture {
    width: usize,
    height: usize,
    color: (f32, f32, f32, f32),
}

impl ScreenTexture {
    pub fn new(width: usize, height: usize, color: (f32, f32, f32, f32)) -> ScreenTexture {
        ScreenTexture {
            width,
            height,
            color,
        }
    }
    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }
}
impl RenderTarget for ScreenTexture {
    fn bind_buffer(&self, context: &Gl) {
        context.viewport(0, 0, self.width as i32, self.height as i32);
        context.bind_framebuffer(Gl::FRAMEBUFFER, None);
    }

    fn clear(&self, context: &Gl) {
        self.bind_buffer(context);
        context.clear_color(self.color.0, self.color.1, self.color.2, self.color.3);
        context.clear(Gl::COLOR_BUFFER_BIT);
    }

    fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}
