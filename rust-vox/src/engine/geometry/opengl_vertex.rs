// Represents a vertex type used by geometry and easily usable with opengl operations

use crate::engine::renderer::opengl_abstractions::vertex_array::VertexLayout;

// Note: use the below line to make a struct implementing a vertex behave like a C struct
// members in rust structs may not be ordered as defined, among other things
// #[repr(C,packed)]


pub trait OpenglVertex
{
    /// Gets the data layout of the vertex
    /// Used by OpenGL when uploading vertex data
    fn get_layout() -> VertexLayout;
}