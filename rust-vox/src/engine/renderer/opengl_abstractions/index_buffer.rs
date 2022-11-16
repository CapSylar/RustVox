use std::mem::size_of;

/// Abstraction over OpenGL's Index Buffer Object
pub struct IndexBuffer
{
    renderer_id : u32,
    _count: usize,
}

impl IndexBuffer
{
    pub fn new( index_data: &[u32] ) -> Self
    {
        let mut buffer_id = 0;

        unsafe
        {
            gl::GenBuffers(1, &mut buffer_id);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffer_id);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (index_data.len() * size_of::<u32>()) as isize , index_data.as_ptr().cast() , gl::STATIC_DRAW);
        }

        Self{ renderer_id: buffer_id, _count: index_data.len() }
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
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.renderer_id)
        }
    }

    pub fn _unbind()
    {
        unsafe
        {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0); // 0 unbinds the currently bound buffer
        }
    }
}

impl Drop for IndexBuffer
{
    fn drop(&mut self)
    {
        unsafe
        {
            gl::DeleteBuffers(1, &self.renderer_id);
        }
    }
}