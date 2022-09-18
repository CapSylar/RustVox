use crate::engine::mesh::Vertex;


/// Abstraction over OpenGL's Vertex Buffer Object
pub struct VertexBuffer
{
    renderer_id : u32,
}

impl VertexBuffer
{
    pub fn new( size_bytes: usize , vertex_data: &[Vertex] ) -> Self
    {
        let mut buffer_id = 0;

        unsafe
        {
            gl::GenBuffers(1, &mut buffer_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffer_id);
            gl::BufferData(gl::ARRAY_BUFFER, size_bytes as isize , vertex_data.as_ptr().cast() , gl::STATIC_DRAW);
        }

        Self{ renderer_id: buffer_id }
    }

    pub fn delete(&self)
    {
        unsafe
        {
            gl::DeleteBuffers(1, &self.renderer_id as _ );
        }
    }

    pub fn bind(&self)
    {
        unsafe
        {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.renderer_id)
        }
    }

    pub fn unbind()
    {
        unsafe
        {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0); // 0 unbinds the currently bound buffer
        }
    }
}