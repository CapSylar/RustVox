use glam::{IVec2};

use crate::engine::{geometry::{voxel_vertex::VoxelVertex, mesh::Mesh, chunk_mesh::Face}};

use super::voxel_fetcher::VoxelFetcher;

#[derive(PartialEq)]
pub enum MeshingOption
{
    Opaque,
    Transparent,
}

pub trait ChunkMesher
{
    /// Generate the mesh for the chunk
    /// 
    /// Generated Mesh is placed in mesh
    fn generate_mesh(chunk_pos: IVec2, voxels: VoxelFetcher, mesh: &mut Mesh<VoxelVertex>, trans_faces: &mut Vec<Face>);
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
    
#[derive(Clone, Copy)]
pub enum NormalDirection // order is important, since the indices are used to index the normal table in the shader
{
    Posx,Posy,Posz,Negx,Negy,Negz
}

impl NormalDirection
{
    pub fn from_index(index: usize) -> NormalDirection
    {
        match index
        {
            0 => NormalDirection::Posx,
            1 => NormalDirection::Posy,
            2 => NormalDirection::Posz,
            3 => NormalDirection::Negx,
            4 => NormalDirection::Negy,
            5 => NormalDirection::Negz,
            _ => NormalDirection::Posx,
        }
    }

    pub fn opposite(&self) -> NormalDirection
    {
        NormalDirection::from_index(*self as usize + 3)
    }
}

pub enum UVs
{
    LowerLeft, LowerRight, UpperLeft, UpperRight
}