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
    tex_coord: Vec2, // U, V
    normal_index : u8, // byte index into a normal LUT in the shader, 6 possible normal vectors
}

impl VoxelVertex
{
    pub fn new( position: Vec3 , normal_index : u8 , texture_coordinate: Vec2 ) -> Self
    {
        Self { position , tex_coord: texture_coordinate, normal_index  }
    }
}

impl OpenglVertex for VoxelVertex
{
    fn get_layout() -> VertexLayout
    {
        let mut vertex_layout = VertexLayout::new();

        vertex_layout.push_f32(3); // vertex(x,y,z)
        vertex_layout.push_f32(2); // uv coordinates(u,v)
        vertex_layout.push_u8(1); // Normal Index

        vertex_layout
    }
}