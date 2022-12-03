// Default allocator written for the chunks
// Allocate Data in the most Naive way possible
// A VBO, EBO, for each Chunk

use std::{collections::HashMap};
use crate::engine::{geometry::{mesh::Mesh, opengl_vertex::OpenglVertex}, renderer::opengl_abstractions::{vertex_buffer::VertexBuffer, index_buffer::IndexBuffer, vertex_array::VertexArray}};

pub struct DefaultAllocator<T>
{
    // allocations
    allocations: HashMap<u32,VertexArray<T>>,
}

impl<T> DefaultAllocator<T>
    where T: OpenglVertex
{
    pub fn new() -> Self
    {
        Self{allocations: HashMap::new()}
    }

    pub fn alloc(&mut self, mesh: &mut Mesh<T>)
    {
        // create the vertex buffer
        let vertex_buffer = VertexBuffer::new(&mesh.vertices);
        // create the index buffer
        let index_buffer = IndexBuffer::new(&mesh.indices);

        let vao = VertexArray::new(vertex_buffer,&T::get_layout(), index_buffer);
        let token = AllocToken::new(vao.get_id());
        self.allocations.insert(vao.get_id(), vao);

        mesh.alloc_token = Some(token);
    }

    pub fn dealloc(&mut self, allocation: AllocToken)
    {
        // Safety: it is not possible to get an invalid index in the AllocToken
        let _vao = self.allocations.remove(&allocation.index).unwrap();
        // drop takes care of deletion
    }

    pub fn get_vao(&self, allocation: &AllocToken) -> &VertexArray<T>
    {
        self.allocations.get(&allocation.index).unwrap()
    }
}

pub struct AllocToken
{
    index: u32,
}

impl AllocToken
{
    fn new(index: u32) -> Self
    {
        Self{index}
    }
}