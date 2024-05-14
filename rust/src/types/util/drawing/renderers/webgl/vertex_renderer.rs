use std::collections::HashMap;

use wasm_bindgen::JsValue;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject,
};

use crate::util::logging::console;

use super::setup::{compile_shader, link_program};

pub struct VertexRenderer {
    program: WebGlProgram,
    attributes: Vec<AttributeData>,
    buffer: WebGlBuffer,
    vao: WebGlVertexArrayObject,
    buffer_data: Vec<f32>,
    uniforms: HashMap<String, WebGlUniformLocation>,
}
struct AttributeData {
    name: String,
    element_size: i32,
    data_size: usize,
    attribute_location: u32,
}

impl VertexRenderer {
    pub fn new(
        context: &WebGl2RenderingContext,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<VertexRenderer, JsValue> {
        let vert_shader = compile_shader(
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
            &vertex_shader,
        )?;

        let frag_shader = compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            &fragment_shader,
        )?;

        let program = link_program(&context, &vert_shader, &frag_shader)?;
        context.use_program(Some(&program));

        let buffer = context.create_buffer().ok_or("Failed to create buffer")?;
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        let vao = context
            .create_vertex_array()
            .ok_or("Could not create vertex array object")?;
        Ok(VertexRenderer {
            program,
            vao,
            buffer,
            attributes: Vec::new(),
            buffer_data: Vec::new(),
            uniforms: HashMap::new(),
        })
    }
    pub fn set_data(
        &mut self,
        context: &WebGl2RenderingContext,
        name: &str,
        data: &[f32],
        element_size: u8,
    ) {
        let mut index = 0;
        for attribute_data in &mut self.attributes {
            if attribute_data.name == name {
                self.buffer_data.splice(
                    index..index + attribute_data.data_size,
                    data.iter().cloned(),
                );
                attribute_data.data_size = data.len();
                attribute_data.element_size = element_size as i32;
                return;
            }
            index += attribute_data.data_size;
        }

        self.buffer_data.extend(data);
        self.attributes.push(AttributeData {
            name: name.to_string(),
            element_size: element_size as i32,
            data_size: data.len(),
            attribute_location: context.get_attrib_location(&self.program, name) as u32,
        });
    }
    pub fn update_data(&mut self, context: &WebGl2RenderingContext) {
        context.bind_vertex_array(Some(&self.vao));
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.buffer));
        // context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.buffer));

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let positions_array_buf_view = js_sys::Float32Array::view(&self.buffer_data);

            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &positions_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        let mut index = 0;
        for attribute_data in &self.attributes {
            context.vertex_attrib_pointer_with_i32(
                attribute_data.attribute_location,
                attribute_data.element_size,
                WebGl2RenderingContext::FLOAT,
                false,
                0,
                4 * index, // 4 * to convert to bytes
            );
            context.enable_vertex_attrib_array(attribute_data.attribute_location as u32);
            index += attribute_data.data_size as i32;
        }
    }
    pub fn set_uniform(
        &mut self,
        context: &WebGl2RenderingContext,
        name: &str,
        set: impl Fn(Option<&WebGlUniformLocation>) -> (),
    ) {
        context.use_program(Some(&self.program));
        if let Some(uniform_location) = self.uniforms.get(name) {
            set(Some(uniform_location));
        } else if let Some(uniform_location) = context.get_uniform_location(&self.program, name) {
            set(Some(&uniform_location));
            self.uniforms.insert(name.to_string(), uniform_location);
        }
    }
    pub fn render(&self, context: &WebGl2RenderingContext, mode: u32) {
        if let Some(attribute) = self.attributes.get(0) {
            context.use_program(Some(&self.program));
            context.bind_vertex_array(Some(&self.vao));
            let points = (attribute.data_size as i32) / attribute.element_size;
            context.draw_arrays(mode, 0, points);
        }
    }

    pub fn dispose(&self, context: &WebGl2RenderingContext) {
        context.delete_buffer(Some(&self.buffer));
        context.delete_program(Some(&self.program));
        context.delete_vertex_array(Some(&self.vao));
    }
}

// impl Drop for VertexRenderer {
//     fn drop(&mut self) {
//     }
// }
