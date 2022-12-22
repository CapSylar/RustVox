use std::{marker::PhantomData, mem::size_of, ffi::c_void};

/// Abstraction over OpenGL's Vertex Buffer Object
pub struct VertexBuffer<T>
{
    renderer_id : u32,
    _phantom: PhantomData<T>,
}

impl<T> VertexBuffer<T>
{
    pub fn new(vertex_data: &[T]) -> Self
    {
        let mut buffer_id = 0;

        unsafe
        {
            gl::GenBuffers(1, &mut buffer_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer_id);
            gl::BufferData(gl::ARRAY_BUFFER, (vertex_data.len() * size_of::<T>()) as isize , vertex_data.as_ptr().cast() , gl::STATIC_DRAW);
        }

        Self{ _phantom: PhantomData, renderer_id: buffer_id }
    }

    /// get the handle to an VertexBuffer and manage it
    /// 
    /// In this case the vertex buffer has already been created and filled externally
    pub fn from_id(renderer_id: u32) -> Self
    {
        Self{_phantom: PhantomData, renderer_id}
    }

    pub fn respecify(&self,vertex_data: &[T])
    {
        // Buffer Respecification (Orphaning)
        unsafe
        {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.renderer_id);
            gl::BufferData(gl::ARRAY_BUFFER, (vertex_data.len() * size_of::<T>()) as isize, std::ptr::null::<c_void>(), gl::STREAM_DRAW);
            gl::BufferData(gl::ARRAY_BUFFER, (vertex_data.len() * size_of::<T>()) as isize , vertex_data.as_ptr().cast() , gl::STREAM_DRAW);
        }
    }

    pub fn _delete(&self)
    {
        unsafe
        {
            gl::DeleteBuffers(1, &self.renderer_id);
        }
    }

    pub fn bind(&self)
    {
        unsafe
        {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.renderer_id)
        }
    }

    pub fn _unbind()
    {
        unsafe
        {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0); // 0 unbinds the currently bound buffer
        }
    }
}

impl<T> Drop for VertexBuffer<T>
{
    fn drop(&mut self)
    {
        unsafe
        {
            gl::DeleteBuffers(1, &self.renderer_id);
        }
    }
}