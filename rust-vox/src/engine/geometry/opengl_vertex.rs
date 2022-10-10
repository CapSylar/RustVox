// Represents a vertex type used by geometry and easily usable with opengl operations

use crate::engine::renderer::opengl_abstractions::vertex_array::VertexLayout;

pub trait OpenglVertex
{
    /// Gets the data layout of the vertex
    /// Used by OpenGL when uploading vertex data
    fn get_layout() -> VertexLayout;
}