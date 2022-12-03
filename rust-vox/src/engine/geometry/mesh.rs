use std::mem::size_of;
use crate::engine::renderer::{allocators::default_allocator::AllocToken};
use super::opengl_vertex::OpenglVertex;

/// Contains everything we need to render geometry to the screen, namely the actual *vertices* and indices which
/// indicate how to construct triangles from the vertices
pub struct Mesh<T>
{
    pub vertices: Vec<T>,
    pub indices: Vec<u32>,
    pub alloc_token: Option<AllocToken>,

    num_triangles: usize
}

impl<T> Mesh<T>
    where T: OpenglVertex
{
    pub fn default() -> Self
    {
        Self{vertices:Vec::new(),indices:Vec::new(),alloc_token:None,num_triangles:0}
    }


    // pub fn upload(&mut self)
    // {
    //     // create the vertex buffer
    //     let vertex_buffer = VertexBuffer::new(&self.vertices);
    //     // create the index buffer
    //     let index_buffer = IndexBuffer::new(&self.indices);

    //     let vao = VertexArray::new(vertex_buffer,&T::get_layout(), index_buffer);
    //     self.vao = Some(vao);
    // }

    /// Delete the Geometry from GPU memory
    pub fn delete_gpu_storage(&mut self)
    {
        
    }

    // pub fn respecify_vertices<F>( &mut self,func: F )
    //     where F: FnOnce(&mut Vec<T>)
    // {
    //     func(&mut self.vertices); // change the vertices
        
    //     if let Some(vao) = &self.
    //     {
    //         vao.vbo.respecify(&self.vertices);
    //     }
    // }

    /// add a vertex to the mesh
    /// 
    /// Note: a subsequent call to methods of the name add_*_indices must the used to build geometry
    /// using added vertices
    pub fn add_vertex(&mut self, vertex: T) -> usize
    {
        self.vertices.push(vertex);
        self.vertices.len() -1
    }

    /// add a triangle to the mesh by specifying the index of vertices already added to the mesh
    pub fn add_triangle_indices(&mut self, p1: usize , p2:usize , p3:usize )
    {
        // Note: the indices must be converted to u32 since they are then fed to Opengl which requires
        // this data type even though usize is the way to go
        self.indices.push(p1 as u32);
        self.indices.push(p2 as u32);
        self.indices.push(p3 as u32);

        self.num_triangles += 1;
    }

    /// add a quad to the mesh by specifying the index of vertices already added to the mesh
    pub fn add_quad_indices(&mut self, p1: usize, p2: usize, p3: usize, p4: usize)
    {
        self.indices.push(p1 as u32);
        self.indices.push(p2 as u32);
        self.indices.push(p3 as u32);
        self.indices.push(p4 as u32);
    }

    /// add a quad to the mesh by providing the vertices
    pub fn _add_triangle(&mut self, p1: T, p2: T, p3:T)
    {
        self.vertices.push(p1);
        self.vertices.push(p2);
        self.vertices.push(p3);

        let first = self.vertices.len()-3; // index of first added vertex

        // construct the triangle (Clock-Wise order)
        self.add_triangle_indices(first, first+1, first+2);
    }

    /// add a quad to the mesh by providing the vertices.
    pub fn add_quad(&mut self, lower_left: T, upper_left: T, upper_right: T, lower_right: T)
    {
        self.vertices.push(lower_left);
        self.vertices.push(upper_left);
        self.vertices.push(upper_right);
        self.vertices.push(lower_right);

        let first = self.vertices.len()-4; // index of first added
        
        // construct the two triangles (Clock-Wise order)
        self.add_triangle_indices(first, first+1, first+2);
        self.add_triangle_indices(first, first+2, first+3);
    }

    pub fn get_vertices_size_bytes(&self) -> usize
    {
        self.vertices.len() * size_of::<T>()
    }

    pub fn get_indices_size_bytes(&self) -> usize
    {
        self.indices.len() * size_of::<u32>()
    }

    pub fn get_num_triangles(&self) -> usize
    {
        self.num_triangles
    }

    pub fn get_num_vertices(&self) -> usize
    {
        self.vertices.len()
    }

}