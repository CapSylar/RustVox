use std::{mem::size_of, ffi::c_void};
use super::vertex_buffer::VertexBuffer;

struct VertexBufferLayoutElement
{
    element_type: u32,
    count: usize,
    normalized: u8,
    size_bytes: usize,
}   

pub struct VertexBufferLayout
{
    elements: Vec<VertexBufferLayoutElement>,
    stride_bytes: usize,
}

impl VertexBufferLayout
{
    pub fn new() -> Self
    {
        Self{elements: Vec::new(),stride_bytes:0}
    }

    pub fn push_f32(&mut self, count: usize)
    {
        let element = VertexBufferLayoutElement { element_type: gl::FLOAT, count , normalized: gl::FALSE , size_bytes: size_of::<f32>() * count};
        self.stride_bytes += element.size_bytes;
        self.elements.push(element);
    }

    pub fn push_unsigned(&mut self, count: usize)
    {
        let element = VertexBufferLayoutElement { element_type: gl::UNSIGNED_INT, count , normalized: gl::FALSE , size_bytes: size_of::<u32>() * count};
        self.stride_bytes += element.size_bytes;
        self.elements.push(element);
    }
}

pub struct VertexArray
{
    renderer_id : u32,
}

impl VertexArray
{
    pub fn new() -> Self
    {
        let mut vao = 0;
        unsafe
        {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }

        Self{renderer_id:vao}
    }

    pub fn add_buffer(&mut self, vertex_buffer: &VertexBuffer , layout: &VertexBufferLayout)
    {
        // setup
        self.bind();
        vertex_buffer.bind();
        let mut offset: usize = 0;
        let mut attrib_index = 0;

        for element in &layout.elements
        {
            unsafe
            {
                gl::VertexAttribPointer(attrib_index, element.count as _ , element.element_type,
                     element.normalized, layout.stride_bytes.try_into().unwrap()  , offset as *const c_void );
                gl::EnableVertexAttribArray(attrib_index);
            }
            attrib_index += 1;
            offset += element.size_bytes;
        };
    }

    pub fn bind(&self)
    {
        unsafe
        {
            gl::BindVertexArray(self.renderer_id);
        }
    }

    pub fn unbind()
    {
        unsafe
        {
            gl::BindVertexArray(0);
        }
    }


}