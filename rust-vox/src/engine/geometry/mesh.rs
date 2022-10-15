use std::mem::size_of;
use crate::engine::renderer::opengl_abstractions::{vertex_array::VertexArray, vertex_buffer::VertexBuffer, index_buffer::IndexBuffer};
use super::opengl_vertex::OpenglVertex;

/// Contains everything we need to render geometry to the screen, namely the actual *vertices* and indices which
/// indicate how to construct triangles from the vertices
pub struct Mesh<T>
{
    pub vertices: Vec<T>,
    pub indices: Vec<u32>,
    pub vao: Option<VertexArray<T>>,
}

impl<T> Mesh<T>
    where T: OpenglVertex
{
    pub fn new() -> Self
    {
        Self{vertices:Vec::new(),indices:Vec::new(),vao:None}
    }

    /// `Upload Mesh to the GPU`
    /// Creates the VAO,VBO and EBO needed render this chunk
    pub fn upload(&mut self)
    {
        // create the vertex buffer
        let vertex_buffer = VertexBuffer::new(&self.vertices);
        // create the index buffer
        let index_buffer = IndexBuffer::new(&self.indices);

        let vao = VertexArray::new(vertex_buffer,&T::get_layout(), index_buffer);
        self.vao = Some(vao);
    }

    pub fn respecify_vertices<F>( &mut self,func: F )
        where F: FnOnce(&mut Vec<T>)
    {
        func(&mut self.vertices); // change the vertices
        
        if let Some(vao) = &self.vao
        {
            vao.vbo.respecify(&self.vertices);
        }
    }

    pub fn add_vertex(&mut self, vertex: T) -> u32
    {
        self.vertices.push(vertex);
        self.vertices.len() as u32 -1
    }

    pub fn add_triangle(&mut self, p1: u32 , p2:u32 , p3:u32 )
    {
        self.indices.push(p1);
        self.indices.push(p2);
        self.indices.push(p3);
    }

    pub fn add_quad(&mut self, p1: u32, p2: u32, p3: u32, p4: u32)
    {
        self.indices.push(p1);
        self.indices.push(p2);
        self.indices.push(p3);
        self.indices.push(p4);
    }

    pub fn size_bytes(&self) -> usize
    {
        self.vertices.len() * size_of::<T>()
    }

}