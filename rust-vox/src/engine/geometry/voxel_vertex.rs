use glam::{Vec3, Vec2};

use crate::engine::renderer::opengl_abstractions::vertex_array::VertexLayout;

use super::opengl_vertex::OpenglVertex;

// Voxel Vertex
/// Holds all the data which constitute a *vertex*
#[repr(C,packed)]
#[derive(Clone,Copy,Debug)]
pub struct VoxelVertex
{
    position: Vec3, // X, Y, Z
    texture_u: u8, // U
    texture_v: u8, // V
    texture_index: u8, // what texture to display
    normal_index : u8, // byte index into a normal LUT in the shader, 6 possible normal vectors
}

impl VoxelVertex
{
    pub fn new( position: Vec3 , normal_index : u8 , texture_uv: (u8,u8), texture_index: u8 ) -> Self
    {
        Self { position , texture_u: texture_uv.0, texture_v: texture_uv.1, normal_index, texture_index }
    }
}

impl OpenglVertex for VoxelVertex
{
    fn get_layout() -> VertexLayout
    {
        let mut vertex_layout = VertexLayout::new();

        vertex_layout.push_f32(3); // vertex(x,y,z)
        vertex_layout.push_u8(1); // texture U
        vertex_layout.push_u8(1); // texture V
        vertex_layout.push_u8(1); // texture index
        vertex_layout.push_u8(1); // Normal Index

        vertex_layout
    }
}