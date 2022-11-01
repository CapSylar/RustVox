use glam::{Vec2, const_vec2};
use crate::engine::{chunk::{CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z, Chunk}, geometry::{voxel_vertex::VoxelVertex, mesh::Mesh}};

pub trait ChunkMesher
{
    /// generate the mesh for the chunk
    fn generate_mesh(chunk: &Chunk) -> Mesh<VoxelVertex>;
}

pub const VOXEL_SIZE: f32 = 1.0;

/// Holds the texture UV information for each block type
/// Each entry holds the lower left UV coordinates of the texture for the face
// #[derive(Clone, Copy)]
// pub struct VoxelTypeTexture
// {
//     pub top_face: Vec2,
//     pub bottom_face: Vec2,
//     pub front_face: Vec2,
//     pub back_face: Vec2,
//     pub right_face: Vec2,
//     pub left_face: Vec2,
// }

// pub const VOXEL_UVDATA : [VoxelTypeTexture ; 2] = [
//     VoxelTypeTexture{ // Grass
//     top_face: const_vec2!([0.125,0.875]), back_face: const_vec2!([0.0,0.875]),
//     bottom_face: const_vec2!([0.25,0.875]), front_face: const_vec2!([0.0,0.875]),
//     left_face: const_vec2!([0.0,0.875]), right_face: const_vec2!([0.0,0.875])},
//     VoxelTypeTexture{ // Sand
//     top_face: const_vec2!([0.375,0.875]), back_face: const_vec2!([0.375,0.875]),
//     bottom_face: const_vec2!([0.375,0.875]), front_face: const_vec2!([0.375,0.875]),
//     left_face: const_vec2!([0.375,0.875]), right_face: const_vec2!([0.375,0.875])}
//     ];

// pub enum VoxelTextureIndex
// {
//     Dirt,
//     Sand
// }

pub const VOXEL_FACE_VALUES : [(i32,i32,i32);6] = 
[
    (0,1,0),
    (0,-1,0),
    (0,0,1),
    (0,0,-1),
    (1,0,0),
    (-1,0,0)
];
    
pub enum Normals
{
    Posy,Negy,Posz,Negz,Posx,Negx
}

pub enum UVs
{
    LowerLeft, LowerRight, UpperLeft, UpperRight
}