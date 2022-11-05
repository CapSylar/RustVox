use glam::{Vec2, const_vec2};
use crate::engine::{chunk::{CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z, Chunk}, geometry::{voxel_vertex::VoxelVertex, mesh::Mesh}};

pub trait ChunkMesher
{
    /// generate the mesh for the chunk
    fn generate_mesh(chunk: &Chunk) -> Mesh<VoxelVertex>;
}

pub const VOXEL_SIZE: f32 = 1.0;

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