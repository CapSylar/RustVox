use std::{mem::size_of, ffi::c_void};
use super::{vertex_buffer::VertexBuffer, index_buffer::IndexBuffer};

pub struct VertexArray
{
    renderer_id : u32, // vao ID
    vbo : VertexBuffer,
    ebo: IndexBuffer,
}

impl VertexArray
{
    pub fn new(vertex_buffer: VertexBuffer, vertex_layout: &VertexBufferLayout, index_buffer: IndexBuffer) -> Self
    {
        let mut vao = 0;
        unsafe
        {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }

        // add the buffer, binds the vertex buffer implicitely
        VertexArray::add_buffer(&vertex_buffer, &vertex_layout);
        // bind the index buffer
        index_buffer.bind();
        // unbind the vao
        unsafe { gl::BindVertexArray(0); } 
        
        Self{renderer_id:vao, vbo:vertex_buffer, ebo:index_buffer}
    }

    //TODO: Document
    fn add_buffer(vertex_buffer: &VertexBuffer , layout: &VertexBufferLayout)
    {
        // setup
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